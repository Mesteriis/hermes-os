use sqlx::{Postgres, Row, Transaction};

use crate::engines::risk::{RiskEngine, RiskSeverity, RiskSignal};

use super::errors::PersonaTrustError;

pub(super) async fn sync_persona_health_status_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    person_id: &str,
) -> Result<(), PersonaTrustError> {
    let rows = sqlx::query(
        r#"
        SELECT severity
        FROM persona_risks
        WHERE person_id = $1
          AND resolved_at IS NULL
        "#,
    )
    .bind(person_id)
    .fetch_all(&mut **transaction)
    .await?;
    let risks = rows
        .into_iter()
        .map(|row| {
            let severity: String = row.try_get("severity")?;
            Ok(RiskSignal::unresolved(RiskSeverity::parse(&severity)?))
        })
        .collect::<Result<Vec<_>, PersonaTrustError>>()?;
    let health_status = RiskEngine::derive_attention_status(&risks).as_persona_health_status();

    sqlx::query(
        "UPDATE personas
         SET health_status = $2, last_health_check = now(), updated_at = now()
         WHERE person_id = $1",
    )
    .bind(person_id)
    .bind(health_status)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
