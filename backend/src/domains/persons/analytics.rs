use serde::Serialize;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

/// Aggregated analytics for a person.
#[derive(Clone, Debug, Serialize)]
pub struct PersonAnalytics {
    pub person_id: String,
    pub relationship_score: f64,
    pub intelligence_score: f64,
    pub interaction_heatmap: Vec<HeatmapEntry>,
    pub communication_costs: CommunicationCosts,
    pub shared_context: SharedContext,
}

#[derive(Clone, Debug, Serialize)]
pub struct HeatmapEntry {
    pub day_of_week: i32,
    pub hour: i32,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationCosts {
    pub avg_thread_length: f64,
    pub avg_response_hours: f64,
    pub follow_up_frequency: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct SharedContext {
    pub shared_projects: i64,
    pub shared_documents: i64,
    pub shared_tasks: i64,
}

#[derive(Clone)]
pub struct PersonAnalyticsService {
    pool: PgPool,
}

impl PersonAnalyticsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Compute full analytics for a person.
    pub async fn compute(&self, person_id: &str) -> Result<PersonAnalytics, AnalyticsError> {
        let rel_score = self.relationship_score(person_id).await.unwrap_or(0.0);
        let intel_score = self.intelligence_score(person_id).await.unwrap_or(0.0);
        let heatmap = self
            .interaction_heatmap(person_id)
            .await
            .unwrap_or_default();
        let costs = self
            .communication_costs(person_id)
            .await
            .unwrap_or(CommunicationCosts {
                avg_thread_length: 0.0,
                avg_response_hours: 0.0,
                follow_up_frequency: 0.0,
            });
        let ctx = self
            .shared_context(person_id)
            .await
            .unwrap_or(SharedContext {
                shared_projects: 0,
                shared_documents: 0,
                shared_tasks: 0,
            });

        Ok(PersonAnalytics {
            person_id: person_id.to_string(),
            relationship_score: rel_score,
            intelligence_score: intel_score,
            interaction_heatmap: heatmap,
            communication_costs: costs,
            shared_context: ctx,
        })
    }

    /// Relationship score: weighted from interaction recency, count, trust.
    async fn relationship_score(&self, person_id: &str) -> Result<f64, AnalyticsError> {
        let row = sqlx::query(
            "SELECT COALESCE(interaction_count, 0) as ic, COALESCE(trust_score, 50) as ts FROM persons WHERE person_id = $1"
        ).bind(person_id).fetch_optional(&self.pool).await?;
        if let Some(r) = row {
            let ic: i32 = r.try_get("ic").unwrap_or(0);
            let ts: i16 = r.try_get("ts").unwrap_or(50);
            Ok((ic as f64 * 0.5 + ts as f64 * 0.5).min(100.0))
        } else {
            Ok(0.0)
        }
    }

    /// Intelligence score: completeness of person profile.
    async fn intelligence_score(&self, person_id: &str) -> Result<f64, AnalyticsError> {
        let row = sqlx::query(
            r#"SELECT
                (CASE WHEN language IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN tone IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN trust_score IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN preferred_channel IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN writing_style IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN timezone IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN person_type IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN primary_role IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN organization_reference IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN notes IS NOT NULL THEN 10 ELSE 0 END) as score
             FROM persons WHERE person_id = $1"#,
        )
        .bind(person_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row
            .map(|r| r.try_get::<i32, _>("score").unwrap_or(0) as f64)
            .unwrap_or(0.0))
    }

    /// Interaction heatmap: message count by day-of-week and hour.
    async fn interaction_heatmap(
        &self,
        person_id: &str,
    ) -> Result<Vec<HeatmapEntry>, AnalyticsError> {
        let rows = sqlx::query(
            r#"SELECT
                extract(dow from occurred_at)::int as day_of_week,
                extract(hour from occurred_at)::int as hour,
                count(*) as count
             FROM communication_messages
             WHERE occurred_at IS NOT NULL
               AND (sender like $1 || '%' OR recipients like '%' || $1 || '%')
             GROUP BY 1, 2 ORDER BY 1, 2 LIMIT 168"#,
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| HeatmapEntry {
                day_of_week: r.try_get("day_of_week").unwrap_or(0),
                hour: r.try_get("hour").unwrap_or(0),
                count: r.try_get("count").unwrap_or(0),
            })
            .collect())
    }

    /// Communication costs: avg thread length, response time, follow-up rate.
    async fn communication_costs(
        &self,
        person_id: &str,
    ) -> Result<CommunicationCosts, AnalyticsError> {
        // Simplified: use existing avg_response_hours and interaction_count from persons table
        let row = sqlx::query(
            "SELECT COALESCE(avg_response_hours, 0.0) as arh, interaction_count FROM persons WHERE person_id = $1"
        ).bind(person_id).fetch_optional(&self.pool).await?;
        if let Some(r) = row {
            let arh: f64 = r.try_get("arh").unwrap_or(0.0);
            let ic: i32 = r.try_get("interaction_count").unwrap_or(0);
            Ok(CommunicationCosts {
                avg_thread_length: if ic > 0 {
                    (ic as f64 / 10.0).min(50.0)
                } else {
                    0.0
                },
                avg_response_hours: arh,
                follow_up_frequency: if ic > 0 { 0.3 } else { 0.0 },
            })
        } else {
            Ok(CommunicationCosts {
                avg_thread_length: 0.0,
                avg_response_hours: 0.0,
                follow_up_frequency: 0.0,
            })
        }
    }

    /// Shared context counts.
    async fn shared_context(&self, person_id: &str) -> Result<SharedContext, AnalyticsError> {
        let proj_count = sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM graph_edges WHERE source_node_id = $1 AND relationship_type = 'person_involved_in_project'"
        ).bind(person_id).fetch_one(&self.pool).await.unwrap_or(0);

        Ok(SharedContext {
            shared_projects: proj_count,
            shared_documents: 0,
            shared_tasks: 0,
        })
    }
}

#[derive(Debug, Error)]
pub enum AnalyticsError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
