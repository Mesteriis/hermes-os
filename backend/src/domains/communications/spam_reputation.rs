use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::domains::communications::messages::ProjectedMessage;

const SCORE_INITIAL: i16 = 100;
const SCORE_SPAM_PENALTY: i16 = 50;
const SCORE_NON_SPAM_RECOVERY: i16 = 10;
const SUPPRESSION_TTL_DAYS: i64 = 30;

#[derive(Clone)]
pub struct SenderReputationStore {
    pool: PgPool,
}

pub type SenderReputationPort = SenderReputationStore;

impl SenderReputationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn evaluate_message(
        &self,
        message: &ProjectedMessage,
    ) -> Result<SenderReputationDecision, SenderReputationError> {
        let key = SenderReputationKey::from_sender(&message.sender)?;
        let record = self.reputation(&key).await?;
        let known_contact = self
            .sender_has_confirmed_person_link(&key.sender_key)
            .await?;
        let suppressed = record.as_ref().is_some_and(|record| {
            record.score == 0
                && !known_contact
                && record
                    .suppressed_until
                    .is_none_or(|until| until > Utc::now())
        });

        Ok(SenderReputationDecision {
            key,
            score: record.as_ref().map_or(SCORE_INITIAL, |record| record.score),
            suppressed,
            known_contact_bypass: record.as_ref().is_some_and(|record| record.score == 0)
                && known_contact,
            reason: suppressed.then(|| "sender_reputation_zero".to_owned()),
        })
    }

    pub async fn record_analysis(
        &self,
        message: &ProjectedMessage,
        classification: SenderReputationClassification,
        reason: &str,
    ) -> Result<SenderReputationRecord, SenderReputationError> {
        let key = SenderReputationKey::from_sender(&message.sender)?;
        let current = self.reputation(&key).await?;
        let current_score = current
            .as_ref()
            .map_or(SCORE_INITIAL, |record| record.score);
        let next_score = match classification {
            SenderReputationClassification::Spam => {
                current_score.saturating_sub(SCORE_SPAM_PENALTY).max(0)
            }
            SenderReputationClassification::NonSpam => current_score
                .saturating_add(SCORE_NON_SPAM_RECOVERY)
                .min(100),
        };
        let suppressed_until =
            (next_score == 0).then(|| Utc::now() + Duration::days(SUPPRESSION_TTL_DAYS));
        let spam_delta = if matches!(classification, SenderReputationClassification::Spam) {
            1
        } else {
            0
        };
        let non_spam_delta = if matches!(classification, SenderReputationClassification::NonSpam) {
            1
        } else {
            0
        };

        let row = sqlx::query(
            r#"
            INSERT INTO communication_sender_reputation (
                sender_key,
                sender_domain,
                score,
                spam_count,
                non_spam_count,
                suppressed_until,
                last_reason,
                last_message_id,
                metadata,
                first_seen_at,
                last_seen_at,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now(), now(), now(), now())
            ON CONFLICT (sender_key, sender_domain)
            DO UPDATE SET
                score = EXCLUDED.score,
                spam_count = communication_sender_reputation.spam_count + $4,
                non_spam_count = communication_sender_reputation.non_spam_count + $5,
                suppressed_until = EXCLUDED.suppressed_until,
                last_reason = EXCLUDED.last_reason,
                last_message_id = EXCLUDED.last_message_id,
                metadata = communication_sender_reputation.metadata || EXCLUDED.metadata,
                last_seen_at = now(),
                updated_at = now()
            RETURNING
                sender_key,
                sender_domain,
                score,
                spam_count,
                non_spam_count,
                suppressed_until,
                last_reason,
                last_message_id,
                metadata,
                first_seen_at,
                last_seen_at,
                created_at,
                updated_at
            "#,
        )
        .bind(&key.sender_key)
        .bind(&key.sender_domain)
        .bind(next_score)
        .bind(spam_delta)
        .bind(non_spam_delta)
        .bind(suppressed_until)
        .bind(reason.trim())
        .bind(&message.message_id)
        .bind(json!({
            "last_classification": classification.as_str(),
            "last_message_observation_id": message.observation_id,
        }))
        .fetch_one(&self.pool)
        .await?;

        row_to_reputation_record(row)
    }

    pub async fn record_suppressed_message(
        &self,
        message: &ProjectedMessage,
        reason: &str,
    ) -> Result<(), SenderReputationError> {
        let key = SenderReputationKey::from_sender(&message.sender)?;
        sqlx::query(
            r#"
            UPDATE communication_sender_reputation
            SET
                last_reason = $3,
                last_message_id = $4,
                last_seen_at = now(),
                updated_at = now(),
                metadata = metadata || $5
            WHERE sender_key = $1 AND sender_domain = $2
            "#,
        )
        .bind(&key.sender_key)
        .bind(&key.sender_domain)
        .bind(reason.trim())
        .bind(&message.message_id)
        .bind(json!({
            "last_suppressed_message_observation_id": message.observation_id,
        }))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn reputation(
        &self,
        key: &SenderReputationKey,
    ) -> Result<Option<SenderReputationRecord>, SenderReputationError> {
        let row = sqlx::query(
            r#"
            SELECT
                sender_key,
                sender_domain,
                score,
                spam_count,
                non_spam_count,
                suppressed_until,
                last_reason,
                last_message_id,
                metadata,
                first_seen_at,
                last_seen_at,
                created_at,
                updated_at
            FROM communication_sender_reputation
            WHERE sender_key = $1 AND sender_domain = $2
            "#,
        )
        .bind(&key.sender_key)
        .bind(&key.sender_domain)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_reputation_record).transpose()
    }

    async fn sender_has_confirmed_person_link(
        &self,
        sender_key: &str,
    ) -> Result<bool, SenderReputationError> {
        let known = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM person_identities identity_trace
                WHERE identity_trace.identity_type = 'email'
                  AND lower(identity_trace.identity_value) = $1
                  AND identity_trace.status = 'active'
                  AND identity_trace.person_id IS NOT NULL
                  AND identity_trace.source <> 'email_sync'
                LIMIT 1
            )
            "#,
        )
        .bind(sender_key)
        .fetch_one(&self.pool)
        .await?;
        Ok(known)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SenderReputationKey {
    pub sender_key: String,
    pub sender_domain: String,
}

impl SenderReputationKey {
    pub fn from_sender(sender: &str) -> Result<Self, SenderReputationError> {
        let sender_key = normalize_sender_email(sender)
            .ok_or_else(|| SenderReputationError::InvalidSender(sender.to_owned()))?;
        let sender_domain = sender_key
            .split('@')
            .nth(1)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("unknown")
            .to_owned();
        Ok(Self {
            sender_key,
            sender_domain,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SenderReputationDecision {
    pub key: SenderReputationKey,
    pub score: i16,
    pub suppressed: bool,
    pub known_contact_bypass: bool,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SenderReputationRecord {
    pub sender_key: String,
    pub sender_domain: String,
    pub score: i16,
    pub spam_count: i32,
    pub non_spam_count: i32,
    pub suppressed_until: Option<DateTime<Utc>>,
    pub last_reason: Option<String>,
    pub last_message_id: Option<String>,
    pub metadata: serde_json::Value,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SenderReputationClassification {
    Spam,
    NonSpam,
}

impl SenderReputationClassification {
    fn as_str(self) -> &'static str {
        match self {
            Self::Spam => "spam",
            Self::NonSpam => "non_spam",
        }
    }
}

#[derive(Debug, Error)]
pub enum SenderReputationError {
    #[error("invalid sender address for reputation: {0}")]
    InvalidSender(String),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

fn row_to_reputation_record(row: PgRow) -> Result<SenderReputationRecord, SenderReputationError> {
    Ok(SenderReputationRecord {
        sender_key: row.try_get("sender_key")?,
        sender_domain: row.try_get("sender_domain")?,
        score: row.try_get("score")?,
        spam_count: row.try_get("spam_count")?,
        non_spam_count: row.try_get("non_spam_count")?,
        suppressed_until: row.try_get("suppressed_until")?,
        last_reason: row.try_get("last_reason")?,
        last_message_id: row.try_get("last_message_id")?,
        metadata: row.try_get("metadata")?,
        first_seen_at: row.try_get("first_seen_at")?,
        last_seen_at: row.try_get("last_seen_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn normalize_sender_email(sender: &str) -> Option<String> {
    let value = sender.trim();
    let candidate = if let Some((_, tail)) = value.rsplit_once('<') {
        tail.split_once('>').map_or(tail, |(email, _)| email)
    } else {
        value
    };
    let email = candidate
        .trim()
        .trim_matches(|c| matches!(c, '"' | '\'' | '<' | '>' | ',' | ';'))
        .to_ascii_lowercase();
    (email.contains('@') && !email.starts_with('@') && !email.ends_with('@')).then_some(email)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reputation_key_normalizes_display_sender() {
        let key = SenderReputationKey::from_sender("Ada Lovelace <Ada@Example.COM>")
            .expect("sender should normalize");

        assert_eq!(key.sender_key, "ada@example.com");
        assert_eq!(key.sender_domain, "example.com");
    }

    #[test]
    fn reputation_key_rejects_invalid_sender() {
        assert!(SenderReputationKey::from_sender("not an email").is_err());
    }
}
