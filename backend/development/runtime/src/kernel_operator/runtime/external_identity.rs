//! Development-profile owner binding for an external runtime public key.

use std::path::PathBuf;

use crate::infrastructure::filesystem::acquire_runtime_directory_lock;
use crate::kernel_operator::control::plane::open_development_control_store;
use crate::modules::registration::registry as module_registry;

pub fn run(
    data_dir_override: Option<PathBuf>,
    registration_id: &str,
    public_key_sec1_hex: &str,
) -> Result<(), String> {
    eprintln!(
        "WARNING: development_full_platform_v1 pins a local runtime public key; production runtime-session transport remains closed"
    );
    let public_key_sec1 = decode_public_key(public_key_sec1_hex)?;
    let data_dir =
        crate::infrastructure::filesystem::resolve_data_directory(data_dir_override.clone())?;
    let (runtime_dir, store) = open_development_control_store(data_dir_override)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let registration = module_registry::bind_external_runtime_identity(
        &data_dir,
        &store,
        registration_id,
        public_key_sec1,
    )?;
    println!("module_registration_id={}", registration.registration_id());
    println!("external_runtime_identity_bound=true");
    println!("module_grant_epoch={}", registration.grant_epoch());
    Ok(())
}

fn decode_public_key(value: &str) -> Result<[u8; 65], String> {
    if value.len() != 130 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("external runtime public key has an invalid encoding".to_owned());
    }
    let mut public_key = [0_u8; 65];
    for (index, byte) in public_key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| "external runtime public key has an invalid encoding".to_owned())?;
    }
    if public_key[0] != 0x04 {
        return Err("external runtime public key has an invalid encoding".to_owned());
    }
    Ok(public_key)
}
