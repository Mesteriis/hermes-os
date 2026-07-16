use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use hermes_gateway_protocol::v1::{
    ExportControlStoreResponseV1, GetRecoveryStatusResponseV1, RecoveryControlRequestV1,
    RecoveryControlResponseV1, ShutdownKernelResponseV1, ValidateControlStoreResponseV1,
};
use hermes_runtime_protocol::v1::{
    ControlStoreStatusV1, KernelStateV1, RecoveryStatusV1,
};
use prost::Message;

use crate::control_store_lifecycle::open_validated_control_store;
use crate::filesystem::{ensure_regular_file_or_absent, prepare_owner_private_directory};

const MAX_RECOVERY_FRAME_BYTES: usize = 64 * 1024;
const RECOVERY_IPC_TIMEOUT: Duration = Duration::from_secs(5);
const RECOVERY_SHUTDOWN_POLL: Duration = Duration::from_millis(25);

pub fn serve_recovery_socket(runtime_dir: &Path, store_path: &Path) -> Result<(), String> {
    let socket_path = runtime_dir.join("recovery.sock");
    ensure_regular_file_or_absent(&socket_path, "recovery socket")?;
    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path).map_err(|error| error.to_string())?;
    let socket_cleanup = RecoverySocketCleanup(socket_path.clone());
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))
        .map_err(|error| error.to_string())?;
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    for signal in [
        signal_hook::consts::signal::SIGINT,
        signal_hook::consts::signal::SIGTERM,
    ] {
        signal_hook::flag::register(signal, Arc::clone(&shutdown_requested))
            .map_err(|error| error.to_string())?;
    }
    println!("recovery_socket={}", socket_path.display());

    let result = loop {
        if shutdown_requested.load(Ordering::Acquire) {
            break Ok(());
        }
        let mut stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(RECOVERY_SHUTDOWN_POLL);
                continue;
            }
            Err(error) => break Err(error.to_string()),
        };
        stream
            .set_read_timeout(Some(RECOVERY_IPC_TIMEOUT))
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(Some(RECOVERY_IPC_TIMEOUT))
            .map_err(|error| error.to_string())?;
        let bytes = read_recovery_frame(&mut stream)?;
        let request = RecoveryControlRequestV1::decode(bytes.as_slice())
            .map_err(|_| "invalid recovery IPC request".to_owned())?;
        let action = recovery_response(request, store_path);
        write_recovery_frame(&mut stream, &action.response.encode_to_vec())?;
        if action.shutdown {
            break Ok(());
        }
    };
    drop(socket_cleanup);
    result
}

struct RecoverySocketCleanup(PathBuf);

impl Drop for RecoverySocketCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn read_recovery_frame(stream: &mut impl Read) -> Result<Vec<u8>, String> {
    let frame_length = read_protobuf_varint(stream)?;
    let frame_length =
        usize::try_from(frame_length).map_err(|_| "recovery IPC frame is too large".to_owned())?;
    if frame_length > MAX_RECOVERY_FRAME_BYTES {
        return Err("recovery IPC frame is too large".to_owned());
    }

    let mut bytes = vec![0_u8; frame_length];
    stream
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
}

fn read_protobuf_varint(stream: &mut impl Read) -> Result<u64, String> {
    let mut value = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|error| error.to_string())?;
        value |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("invalid recovery IPC frame length".to_owned())
}

fn write_recovery_frame(stream: &mut impl Write, bytes: &[u8]) -> Result<(), String> {
    let length =
        u32::try_from(bytes.len()).map_err(|_| "recovery IPC response is too large".to_owned())?;
    let mut value = length;
    while value >= 0x80 {
        stream
            .write_all(&[((value as u8 & 0x7f) | 0x80)])
            .map_err(|error| error.to_string())?;
        value >>= 7;
    }
    stream
        .write_all(&[value as u8])
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|error| error.to_string())
}

struct RecoveryAction {
    response: RecoveryControlResponseV1,
    shutdown: bool,
}

fn recovery_response(request: RecoveryControlRequestV1, store_path: &Path) -> RecoveryAction {
    use hermes_gateway_protocol::v1::recovery_control_request_v1::Operation;
    use hermes_gateway_protocol::v1::recovery_control_response_v1::Result;

    match request.operation {
        Some(Operation::GetRecoveryStatus(_)) => RecoveryAction {
            response: RecoveryControlResponseV1 {
                result: Some(Result::GetRecoveryStatus(GetRecoveryStatusResponseV1 {
                    status: Some(recovery_status(store_path)),
                })),
                error_code: String::new(),
            },
            shutdown: false,
        },
        Some(Operation::ValidateControlStore(_)) => RecoveryAction {
            response: RecoveryControlResponseV1 {
                result: Some(Result::ValidateControlStore(
                    ValidateControlStoreResponseV1 {
                        status: Some(recovery_status(store_path)),
                    },
                )),
                error_code: String::new(),
            },
            shutdown: false,
        },
        Some(Operation::ExportControlStore(_)) => export_recovery_control_store(store_path),
        Some(Operation::ShutdownKernel(_)) => RecoveryAction {
            response: RecoveryControlResponseV1 {
                result: Some(Result::ShutdownKernel(ShutdownKernelResponseV1 {})),
                error_code: String::new(),
            },
            shutdown: true,
        },
        None => RecoveryAction {
            response: RecoveryControlResponseV1 {
                result: None,
                error_code: "operation_not_available".to_owned(),
            },
            shutdown: false,
        },
    }
}

fn recovery_status(store_path: &Path) -> RecoveryStatusV1 {
    match open_validated_control_store(store_path) {
        Ok(store) => RecoveryStatusV1 {
            state: KernelStateV1::RecoveryOnly as i32,
            control_store_status: ControlStoreStatusV1::Trustworthy as i32,
            kernel_generation: store.snapshot().generation(),
            blocker_code: String::new(),
        },
        Err(_) => RecoveryStatusV1 {
            state: KernelStateV1::RecoveryOnly as i32,
            control_store_status: ControlStoreStatusV1::Unavailable as i32,
            kernel_generation: 0,
            blocker_code: "control_store_unavailable".to_owned(),
        },
    }
}

fn export_recovery_control_store(store_path: &Path) -> RecoveryAction {
    let Some(data_dir) = store_path.parent() else {
        return unavailable_export_action();
    };
    let export_dir = data_dir.join("recovery");
    if prepare_owner_private_directory(&export_dir).is_err() {
        return unavailable_export_action();
    }
    let destination = export_dir.join("control-store.sqlite");
    let Ok(store) = open_validated_control_store(store_path) else {
        return unavailable_export_action();
    };
    let Ok(export) = store.export_to(&destination) else {
        return unavailable_export_action();
    };
    let Ok(export_size_bytes) = std::fs::metadata(destination).map(|metadata| metadata.len())
    else {
        return unavailable_export_action();
    };

    RecoveryAction {
        response: RecoveryControlResponseV1 {
            result: Some(
                hermes_gateway_protocol::v1::recovery_control_response_v1::Result::ExportControlStore(
                    ExportControlStoreResponseV1 {
                        export_sha256: export.sha256_bytes().to_vec(),
                        export_size_bytes,
                    },
                ),
            ),
            error_code: String::new(),
        },
        shutdown: false,
    }
}

fn unavailable_export_action() -> RecoveryAction {
    RecoveryAction {
        response: RecoveryControlResponseV1 {
            result: None,
            error_code: "control_store_export_unavailable".to_owned(),
        },
        shutdown: false,
    }
}
