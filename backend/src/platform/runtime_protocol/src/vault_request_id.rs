//! Canonical process-local request stream for opaque Vault ciphertext routes.

use std::sync::{
    OnceLock,
    atomic::{AtomicU64, Ordering},
};

const REQUEST_ID_BYTES: usize = 16;
static REQUEST_STREAM_ID: OnceLock<Option<u64>> = OnceLock::new();
static NEXT_REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[must_use]
pub fn next_vault_transport_request_id_v1() -> Option<[u8; REQUEST_ID_BYTES]> {
    let stream_id = *REQUEST_STREAM_ID.get_or_init(random_non_zero_stream_id);
    let sequence = NEXT_REQUEST_SEQUENCE
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .ok()?;
    vault_transport_request_id_v1(stream_id?, sequence)
}

#[must_use]
pub fn vault_transport_request_id_v1(
    stream_id: u64,
    sequence: u64,
) -> Option<[u8; REQUEST_ID_BYTES]> {
    if stream_id == 0 || sequence == 0 {
        return None;
    }
    let mut request_id = [0_u8; REQUEST_ID_BYTES];
    request_id[..8].copy_from_slice(&stream_id.to_be_bytes());
    request_id[8..].copy_from_slice(&sequence.to_be_bytes());
    Some(request_id)
}

#[must_use]
pub fn vault_transport_request_position_v1(
    request_id: &[u8; REQUEST_ID_BYTES],
) -> Option<(u64, u64)> {
    let stream_id = u64::from_be_bytes(request_id[..8].try_into().ok()?);
    let sequence = u64::from_be_bytes(request_id[8..].try_into().ok()?);
    (stream_id > 0 && sequence > 0).then_some((stream_id, sequence))
}

fn random_non_zero_stream_id() -> Option<u64> {
    let mut bytes = [0_u8; 8];
    getrandom::fill(&mut bytes).ok()?;
    let stream_id = u64::from_be_bytes(bytes);
    (stream_id > 0).then_some(stream_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_v1_request_id_round_trips_a_stream_and_big_endian_sequence() {
        let request_id =
            vault_transport_request_id_v1(0x1112_1314_1516_1718, 0x0102_0304_0506_0708)
                .expect("positive stream and sequence");

        assert_eq!(&request_id[..8], &[17, 18, 19, 20, 21, 22, 23, 24]);
        assert_eq!(&request_id[8..], &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(
            vault_transport_request_position_v1(&request_id),
            Some((0x1112_1314_1516_1718, 0x0102_0304_0506_0708))
        );
    }

    #[test]
    fn zero_stream_or_sequence_fails_closed() {
        assert!(vault_transport_request_id_v1(0, 1).is_none());
        assert!(vault_transport_request_id_v1(1, 0).is_none());
        assert!(vault_transport_request_position_v1(&[0_u8; 16]).is_none());
    }

    #[test]
    fn process_allocator_is_strictly_monotonic() {
        let first = next_vault_transport_request_id_v1().expect("first request");
        let second = next_vault_transport_request_id_v1().expect("second request");

        let (first_stream, first_sequence) =
            vault_transport_request_position_v1(&first).expect("first position");
        let (second_stream, second_sequence) =
            vault_transport_request_position_v1(&second).expect("second position");
        assert_eq!(first_stream, second_stream);
        assert!(second_sequence > first_sequence);
    }
}
