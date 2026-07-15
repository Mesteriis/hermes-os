const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

// V1 local boundary fingerprint only. This is deterministic for idempotence but
// is not cryptographic evidence of source content.
pub(super) fn local_markdown_fingerprint(extracted_text: &str) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in extracted_text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("local-v1:markdown:{hash:016x}")
}
