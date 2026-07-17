//! Owner-local relation naming rules.

use pg_query::protobuf::RangeVar;

pub(super) fn valid_owner(owner: &str) -> bool {
    !owner.is_empty()
        && owner
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
}

pub(super) fn is_owned_relation(relation: Option<&RangeVar>, owner: &str) -> bool {
    let Some(relation) = relation else {
        return false;
    };
    relation.catalogname.is_empty()
        && relation.schemaname == "hermes_data"
        && relation.relname.starts_with(&format!("{owner}_"))
}
