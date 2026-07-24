//! Correlated duplex framing for one inherited managed-control stream.
//!
//! This module owns only frame identity, bounded I/O and request/response
//! routing. Operation authorization remains with Kernel and operation meaning
//! remains in the typed protocol messages.

use std::collections::BTreeSet;
use std::io::{Read, Write};

use prost::Message;

use crate::v1::{
    ManagedRuntimeControlFrameV2, ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    managed_runtime_control_frame_v2::Frame,
};
use crate::validation::managed_control::{
    MANAGED_CONTROL_CORRELATION_ID_BYTES, MANAGED_CONTROL_TRANSPORT_MAJOR_V2,
    validate_managed_control_frame_v2,
};

pub const MAX_MANAGED_CONTROL_FRAME_BYTES_V2: usize = 512 * 1024;
pub const MAX_MANAGED_CONTROL_PENDING_REQUESTS_V2: usize = 64;

#[derive(Debug)]
pub enum ManagedControlTransportErrorV2 {
    InvalidCorrelationId,
    InvalidFrame,
    FrameTooLarge,
    Io(std::io::Error),
    UnexpectedResponse,
    UnexpectedRequest,
    DuplicateCorrelationId,
    PendingRequestLimit,
    PeerClosed,
}

impl From<std::io::Error> for ManagedControlTransportErrorV2 {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct ManagedControlChannelV2<S> {
    stream: S,
    pending_request_ids: BTreeSet<[u8; MANAGED_CONTROL_CORRELATION_ID_BYTES]>,
}

impl<S> ManagedControlChannelV2<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            pending_request_ids: BTreeSet::new(),
        }
    }

    pub fn into_inner(self) -> S {
        self.stream
    }
}

impl<S: Read + Write> ManagedControlChannelV2<S> {
    pub fn receive_request(
        &mut self,
    ) -> Result<
        (
            [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
            ManagedRuntimeControlRequestV1,
        ),
        ManagedControlTransportErrorV2,
    > {
        let frame = self.read_frame()?;
        let correlation_id = correlation_id_from_slice(&frame.correlation_id)?;
        match frame.frame {
            Some(Frame::Request(request)) => Ok((correlation_id, request)),
            Some(Frame::Response(_)) => Err(ManagedControlTransportErrorV2::UnexpectedResponse),
            None => Err(ManagedControlTransportErrorV2::InvalidFrame),
        }
    }

    pub fn request<F>(
        &mut self,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: ManagedRuntimeControlRequestV1,
        mut dispatch_request: F,
    ) -> Result<ManagedRuntimeControlResponseV1, ManagedControlTransportErrorV2>
    where
        F: FnMut(
            &mut Self,
            [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
            ManagedRuntimeControlRequestV1,
        ) -> Result<(), ManagedControlTransportErrorV2>,
    {
        self.begin_pending(correlation_id)?;
        if let Err(error) = self.write_request(correlation_id, request) {
            self.pending_request_ids.remove(&correlation_id);
            return Err(error);
        }
        let result = (|| loop {
            let frame = self.read_frame()?;
            let received_id = correlation_id_from_slice(&frame.correlation_id)?;
            match frame.frame {
                Some(Frame::Response(response)) if received_id == correlation_id => {
                    break Ok(response);
                }
                Some(Frame::Response(_)) => {
                    break Err(ManagedControlTransportErrorV2::UnexpectedResponse);
                }
                Some(Frame::Request(request)) => dispatch_request(self, received_id, request)?,
                None => break Err(ManagedControlTransportErrorV2::InvalidFrame),
            }
        })();
        self.pending_request_ids.remove(&correlation_id);
        result
    }

    pub fn write_request(
        &mut self,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: ManagedRuntimeControlRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        self.write_frame(ManagedRuntimeControlFrameV2 {
            transport_major: MANAGED_CONTROL_TRANSPORT_MAJOR_V2,
            correlation_id: correlation_id.to_vec(),
            frame: Some(Frame::Request(request)),
        })
    }

    pub fn write_response(
        &mut self,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        response: ManagedRuntimeControlResponseV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        self.write_frame(ManagedRuntimeControlFrameV2 {
            transport_major: MANAGED_CONTROL_TRANSPORT_MAJOR_V2,
            correlation_id: correlation_id.to_vec(),
            frame: Some(Frame::Response(response)),
        })
    }

    pub fn read_frame(
        &mut self,
    ) -> Result<ManagedRuntimeControlFrameV2, ManagedControlTransportErrorV2> {
        let bytes = read_length_delimited(&mut self.stream)?;
        let frame = ManagedRuntimeControlFrameV2::decode(bytes.as_slice())
            .map_err(|_| ManagedControlTransportErrorV2::InvalidFrame)?;
        validate_managed_control_frame_v2(&frame)
            .map_err(|_| ManagedControlTransportErrorV2::InvalidFrame)?;
        Ok(frame)
    }

    fn write_frame(
        &mut self,
        frame: ManagedRuntimeControlFrameV2,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        validate_managed_control_frame_v2(&frame)
            .map_err(|_| ManagedControlTransportErrorV2::InvalidFrame)?;
        let bytes = frame.encode_to_vec();
        if bytes.len() > MAX_MANAGED_CONTROL_FRAME_BYTES_V2 {
            return Err(ManagedControlTransportErrorV2::FrameTooLarge);
        }
        write_length_delimited(&mut self.stream, &bytes)?;
        Ok(())
    }

    fn begin_pending(
        &mut self,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
    ) -> Result<(), ManagedControlTransportErrorV2> {
        if self.pending_request_ids.len() >= MAX_MANAGED_CONTROL_PENDING_REQUESTS_V2 {
            return Err(ManagedControlTransportErrorV2::PendingRequestLimit);
        }
        if !self.pending_request_ids.insert(correlation_id) {
            return Err(ManagedControlTransportErrorV2::DuplicateCorrelationId);
        }
        Ok(())
    }
}

fn correlation_id_from_slice(
    value: &[u8],
) -> Result<[u8; MANAGED_CONTROL_CORRELATION_ID_BYTES], ManagedControlTransportErrorV2> {
    value
        .try_into()
        .map_err(|_| ManagedControlTransportErrorV2::InvalidCorrelationId)
}

fn read_length_delimited(
    stream: &mut impl Read,
) -> Result<Vec<u8>, ManagedControlTransportErrorV2> {
    let mut length = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte)?;
        length |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            let length = usize::try_from(length)
                .map_err(|_| ManagedControlTransportErrorV2::FrameTooLarge)?;
            if length == 0 {
                return Err(ManagedControlTransportErrorV2::PeerClosed);
            }
            if length > MAX_MANAGED_CONTROL_FRAME_BYTES_V2 {
                return Err(ManagedControlTransportErrorV2::FrameTooLarge);
            }
            let mut bytes = vec![0_u8; length];
            stream.read_exact(&mut bytes)?;
            return Ok(bytes);
        }
    }
    Err(ManagedControlTransportErrorV2::FrameTooLarge)
}

fn write_length_delimited(
    stream: &mut impl Write,
    bytes: &[u8],
) -> Result<(), ManagedControlTransportErrorV2> {
    let mut length =
        u32::try_from(bytes.len()).map_err(|_| ManagedControlTransportErrorV2::FrameTooLarge)?;
    while length >= 0x80 {
        stream.write_all(&[(length as u8 & 0x7f) | 0x80])?;
        length >>= 7;
    }
    stream.write_all(&[length as u8])?;
    stream.write_all(bytes)?;
    stream.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::UnixStream;
    use std::thread;

    use super::*;
    use crate::v1::{
        ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
        managed_runtime_control_request_v1::Operation,
    };

    fn ready_request() -> ManagedRuntimeControlRequestV1 {
        ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::Ready(Default::default())),
        }
    }

    fn rejected_response() -> ManagedRuntimeControlResponseV1 {
        ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: "REJECTED".to_owned(),
        }
    }

    #[test]
    fn concurrent_opposite_direction_requests_keep_their_responses_correlated() {
        let (left, right) = UnixStream::pair().expect("control pair");
        let peer = thread::spawn(move || {
            let mut channel = ManagedControlChannelV2::new(right);
            channel.request(
                [2; MANAGED_CONTROL_CORRELATION_ID_BYTES],
                ready_request(),
                |channel, id, _| channel.write_response(id, rejected_response()),
            )
        });

        let mut channel = ManagedControlChannelV2::new(left);
        let response = channel
            .request(
                [1; MANAGED_CONTROL_CORRELATION_ID_BYTES],
                ready_request(),
                |channel, id, _| channel.write_response(id, rejected_response()),
            )
            .expect("correlated response");

        assert_eq!(response.error_code, "REJECTED");
        assert_eq!(
            peer.join()
                .expect("peer join")
                .expect("peer response")
                .error_code,
            "REJECTED"
        );
    }

    #[test]
    fn rejects_a_duplicate_pending_correlation_id() {
        let (stream, _peer) = UnixStream::pair().expect("control pair");
        let mut channel = ManagedControlChannelV2::new(stream);
        channel
            .begin_pending([7; MANAGED_CONTROL_CORRELATION_ID_BYTES])
            .expect("first pending request");

        assert!(matches!(
            channel.begin_pending([7; MANAGED_CONTROL_CORRELATION_ID_BYTES]),
            Err(ManagedControlTransportErrorV2::DuplicateCorrelationId)
        ));
    }

    #[test]
    fn receives_only_typed_requests_with_their_correlation_id() {
        let (writer, reader) = UnixStream::pair().expect("control pair");
        let writer = thread::spawn(move || {
            let mut channel = ManagedControlChannelV2::new(writer);
            channel
                .write_request([9; MANAGED_CONTROL_CORRELATION_ID_BYTES], ready_request())
                .expect("write request");
        });
        let mut channel = ManagedControlChannelV2::new(reader);
        let (correlation_id, request) = channel.receive_request().expect("receive request");

        assert_eq!(correlation_id, [9; MANAGED_CONTROL_CORRELATION_ID_BYTES]);
        assert!(request.operation.is_some());
        writer.join().expect("writer join");
    }

    #[test]
    fn releases_correlation_when_initial_write_fails() {
        let (stream, peer) = UnixStream::pair().expect("control pair");
        drop(peer);
        let mut channel = ManagedControlChannelV2::new(stream);
        let correlation_id = [3; MANAGED_CONTROL_CORRELATION_ID_BYTES];

        assert!(matches!(
            channel.request(correlation_id, ready_request(), |_, _, _| Ok(())),
            Err(ManagedControlTransportErrorV2::Io(_))
        ));
        assert!(channel.begin_pending(correlation_id).is_ok());
    }
}
