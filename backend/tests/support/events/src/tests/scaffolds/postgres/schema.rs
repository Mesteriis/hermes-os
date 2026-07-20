//! Fixed SQL statements for one explicitly catalogued test owner.

use super::super::OwnerDeliveryScaffoldV1;

pub(super) fn install(scaffold: OwnerDeliveryScaffoldV1) -> String {
    let schema = scaffold.schema_name();
    format!(
        "CREATE SCHEMA IF NOT EXISTS {schema};\
         CREATE TABLE IF NOT EXISTS {schema}.durable_outbox_v1 (\
           outbox_id TEXT PRIMARY KEY,\
           message_id BYTEA NOT NULL UNIQUE,\
           envelope_sha256 BYTEA NOT NULL,\
           exact_envelope BYTEA NOT NULL,\
           delivery_state TEXT NOT NULL CHECK (delivery_state IN ('pending', 'published')),\
           published_stream TEXT,\
           published_sequence BIGINT\
         );\
         CREATE TABLE IF NOT EXISTS {schema}.durable_inbox_v1 (\
           message_id BYTEA PRIMARY KEY,\
           envelope_sha256 BYTEA NOT NULL\
         );"
    )
}

pub(super) fn insert_outbox(scaffold: OwnerDeliveryScaffoldV1) -> String {
    format!(
        "INSERT INTO {}.durable_outbox_v1 \
         (outbox_id, message_id, envelope_sha256, exact_envelope, delivery_state) \
         VALUES ($1, $2, $3, $4, 'pending')",
        scaffold.schema_name()
    )
}

pub(super) fn next_pending(scaffold: OwnerDeliveryScaffoldV1) -> String {
    format!(
        "SELECT outbox_id, message_id, envelope_sha256, exact_envelope \
         FROM {}.durable_outbox_v1 WHERE delivery_state = 'pending' \
         ORDER BY outbox_id LIMIT 1",
        scaffold.schema_name()
    )
}

pub(super) fn mark_published(scaffold: OwnerDeliveryScaffoldV1) -> String {
    format!(
        "UPDATE {}.durable_outbox_v1 SET delivery_state = 'published', \
         published_stream = $2, published_sequence = $3 \
         WHERE outbox_id = $1 AND (delivery_state = 'pending' OR \
           (delivery_state = 'published' AND published_stream = $2 AND published_sequence = $3))",
        scaffold.schema_name()
    )
}

pub(super) fn insert_inbox(scaffold: OwnerDeliveryScaffoldV1) -> String {
    format!(
        "INSERT INTO {}.durable_inbox_v1 (message_id, envelope_sha256) \
         VALUES ($1, $2) ON CONFLICT (message_id) DO NOTHING",
        scaffold.schema_name()
    )
}

pub(super) fn inbox_hash(scaffold: OwnerDeliveryScaffoldV1) -> String {
    format!(
        "SELECT envelope_sha256 FROM {}.durable_inbox_v1 WHERE message_id = $1",
        scaffold.schema_name()
    )
}
