use super::constants::{FNV_OFFSET_BASIS, FNV_PRIME, TASK_CANDIDATE_ID_PREFIX, TASK_ID_PREFIX};

pub(crate) fn task_candidate_id_from_source(
    source_kind: &str,
    source_id: &str,
    title: &str,
) -> String {
    let title_hash = fnv1a64_hex(title);
    format!("{TASK_CANDIDATE_ID_PREFIX}{source_kind}:{source_id}:{title_hash}")
}

pub(crate) fn task_id_from_candidate(task_candidate_id: &str) -> String {
    format!("{TASK_ID_PREFIX}{}", fnv1a64_hex(task_candidate_id))
}

fn fnv1a64_hex(value: &str) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("{hash:016x}")
}
