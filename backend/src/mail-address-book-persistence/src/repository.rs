use sqlx::PgPool;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    WrongContract,
    HashMismatch,
    Conflict,
    NotFound,
    StorageUnavailable,
}

#[derive(Clone)]
pub struct MailAddressBookPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl MailAddressBookPersistenceV1 {
    #[must_use]
    pub fn from_owner_local_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn verify_storage_ready(&self) -> Result<(), MailAddressBookPersistenceErrorV1> {
        sqlx::query(
            "SELECT inbox.command_id, result.message_id
             FROM hermes_data.mail_address_book_upsert_inbox inbox,
                  hermes_data.mail_address_book_upsert_result_outbox result
             WHERE FALSE",
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| MailAddressBookPersistenceErrorV1::StorageUnavailable)
    }
}
