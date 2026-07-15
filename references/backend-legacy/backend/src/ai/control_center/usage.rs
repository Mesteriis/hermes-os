use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;

use crate::ai::hub::{AiHubUsageError, AiHubUsageEvent, AiHubUsageRecorder};

use super::models::{
    AiHubHourlyUsageStats, AiHubProviderUsageStats, AiHubUsageStatsResponse, AiHubUsageTotals,
    AiProviderAccount,
};
use super::store::AiControlCenterStore;

const DEFAULT_USAGE_WINDOW_HOURS: i32 = 24;

#[async_trait]
impl AiHubUsageRecorder for AiControlCenterStore {
    async fn record_ai_hub_usage(&self, event: AiHubUsageEvent) -> Result<(), AiHubUsageError> {
        sqlx::query(
            r#"
            INSERT INTO ai_hub_usage_events (
                usage_event_id,
                provider_id,
                model_key,
                route_slot,
                operation,
                status,
                prompt_chars,
                output_chars,
                estimated_input_tokens,
                estimated_output_tokens,
                total_duration_ns,
                latency_ms,
                error_summary
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(&event.usage_event_id)
        .bind(&event.provider_id)
        .bind(&event.model_key)
        .bind(&event.route_slot)
        .bind(&event.operation)
        .bind(&event.status)
        .bind(event.prompt_chars)
        .bind(event.output_chars)
        .bind(event.estimated_input_tokens)
        .bind(event.estimated_output_tokens)
        .bind(event.total_duration_ns)
        .bind(event.latency_ms)
        .bind(&event.error_summary)
        .execute(&self.pool)
        .await
        .map_err(|error| AiHubUsageError(error.to_string()))?;

        Ok(())
    }
}

impl AiControlCenterStore {
    pub async fn usage_stats(
        &self,
        window_hours: Option<i32>,
    ) -> Result<AiHubUsageStatsResponse, super::errors::AiControlCenterError> {
        let window_hours = window_hours
            .unwrap_or(DEFAULT_USAGE_WINDOW_HOURS)
            .clamp(1, 24 * 30);
        let providers = self.list_providers().await?;
        let provider_rows = self.provider_usage_rows(window_hours).await?;
        let hourly = self.hourly_usage_rows(window_hours).await?;

        let mut provider_stats = Vec::with_capacity(providers.len());
        for provider in providers {
            let usage = provider_rows
                .iter()
                .find(|row| row.provider_id == provider.provider_id);
            provider_stats.push(provider_usage_stats(&provider, usage));
        }

        let totals = usage_totals(&provider_stats);

        Ok(AiHubUsageStatsResponse {
            generated_at: Utc::now(),
            window_hours,
            totals,
            providers: provider_stats,
            hourly,
        })
    }

    async fn provider_usage_rows(
        &self,
        window_hours: i32,
    ) -> Result<Vec<ProviderUsageRow>, super::errors::AiControlCenterError> {
        let rows = sqlx::query(
            r#"
            SELECT
                provider_id,
                count(*)::BIGINT AS request_count,
                count(*) FILTER (WHERE status = 'completed')::BIGINT AS completed_count,
                count(*) FILTER (WHERE status = 'failed')::BIGINT AS failed_count,
                coalesce(sum(estimated_input_tokens + coalesce(estimated_output_tokens, 0)), 0)::BIGINT
                    AS estimated_tokens,
                avg(latency_ms)::DOUBLE PRECISION AS avg_latency_ms,
                max(created_at) AS last_request_at
            FROM ai_hub_usage_events
            WHERE created_at >= now() - make_interval(hours => $1)
              AND provider_id IS NOT NULL
            GROUP BY provider_id
            "#,
        )
        .bind(window_hours)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(ProviderUsageRow {
                    provider_id: row.try_get("provider_id")?,
                    request_count: row.try_get("request_count")?,
                    completed_count: row.try_get("completed_count")?,
                    failed_count: row.try_get("failed_count")?,
                    estimated_tokens: row.try_get("estimated_tokens")?,
                    avg_latency_ms: row.try_get("avg_latency_ms")?,
                    last_request_at: row.try_get("last_request_at")?,
                })
            })
            .collect()
    }

    async fn hourly_usage_rows(
        &self,
        window_hours: i32,
    ) -> Result<Vec<AiHubHourlyUsageStats>, super::errors::AiControlCenterError> {
        let rows = sqlx::query(
            r#"
            SELECT
                date_trunc('hour', created_at) AS hour,
                provider_id,
                count(*)::BIGINT AS request_count,
                count(*) FILTER (WHERE status = 'failed')::BIGINT AS failed_count,
                coalesce(sum(estimated_input_tokens + coalesce(estimated_output_tokens, 0)), 0)::BIGINT
                    AS estimated_tokens
            FROM ai_hub_usage_events
            WHERE created_at >= now() - make_interval(hours => $1)
              AND provider_id IS NOT NULL
            GROUP BY hour, provider_id
            ORDER BY hour ASC, provider_id ASC
            "#,
        )
        .bind(window_hours)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(AiHubHourlyUsageStats {
                    hour: row.try_get::<DateTime<Utc>, _>("hour")?,
                    provider_id: row.try_get("provider_id")?,
                    request_count: row.try_get("request_count")?,
                    failed_count: row.try_get("failed_count")?,
                    estimated_tokens: row.try_get("estimated_tokens")?,
                })
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
struct ProviderUsageRow {
    provider_id: String,
    request_count: i64,
    completed_count: i64,
    failed_count: i64,
    estimated_tokens: i64,
    avg_latency_ms: Option<f64>,
    last_request_at: Option<DateTime<Utc>>,
}

fn provider_usage_stats(
    provider: &AiProviderAccount,
    usage: Option<&ProviderUsageRow>,
) -> AiHubProviderUsageStats {
    let estimated_tokens = usage.map(|row| row.estimated_tokens).unwrap_or(0);
    AiHubProviderUsageStats {
        provider_id: provider.provider_id.clone(),
        provider_kind: provider.provider_kind.clone(),
        provider_key: provider.provider_key.clone(),
        display_name: provider.display_name.clone(),
        status: provider.status.clone(),
        request_count: usage.map(|row| row.request_count).unwrap_or(0),
        completed_count: usage.map(|row| row.completed_count).unwrap_or(0),
        failed_count: usage.map(|row| row.failed_count).unwrap_or(0),
        estimated_tokens,
        estimated_cost_usd: estimated_cost_usd(&provider.config, estimated_tokens),
        avg_latency_ms: usage.and_then(|row| row.avg_latency_ms),
        balance_remaining_usd: numeric_config(&provider.config, "balance_remaining_usd"),
        token_quota_remaining: integer_config(&provider.config, "token_quota_remaining"),
        last_request_at: usage.and_then(|row| row.last_request_at),
    }
}

fn usage_totals(providers: &[AiHubProviderUsageStats]) -> AiHubUsageTotals {
    let request_count = providers.iter().map(|row| row.request_count).sum();
    let completed_count = providers.iter().map(|row| row.completed_count).sum();
    let failed_count = providers.iter().map(|row| row.failed_count).sum();
    let estimated_tokens = providers.iter().map(|row| row.estimated_tokens).sum();
    let estimated_costs: Vec<f64> = providers
        .iter()
        .filter_map(|row| row.estimated_cost_usd)
        .collect();
    let estimated_cost_usd = if estimated_costs.is_empty() {
        None
    } else {
        Some(estimated_costs.iter().sum())
    };
    let weighted_latency_sum: f64 = providers
        .iter()
        .filter_map(|row| {
            row.avg_latency_ms
                .map(|latency| latency * row.request_count as f64)
        })
        .sum();
    let latency_request_count: i64 = providers
        .iter()
        .filter(|row| row.avg_latency_ms.is_some())
        .map(|row| row.request_count)
        .sum();
    let avg_latency_ms = if latency_request_count == 0 {
        None
    } else {
        Some(weighted_latency_sum / latency_request_count as f64)
    };

    AiHubUsageTotals {
        request_count,
        completed_count,
        failed_count,
        estimated_tokens,
        estimated_cost_usd,
        avg_latency_ms,
    }
}

fn estimated_cost_usd(config: &Value, estimated_tokens: i64) -> Option<f64> {
    let cost_per_1k = numeric_config(config, "cost_per_1k_tokens_usd")?;
    Some((estimated_tokens as f64 / 1000.0) * cost_per_1k)
}

fn numeric_config(config: &Value, key: &str) -> Option<f64> {
    config.get(key).and_then(Value::as_f64)
}

fn integer_config(config: &Value, key: &str) -> Option<i64> {
    config.get(key).and_then(Value::as_i64)
}
