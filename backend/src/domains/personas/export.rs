use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::personas::investigator::errors::InvestigatorError;
use crate::domains::personas::investigator::models::PersonaDossier;
use crate::domains::personas::investigator::service::PersonaInvestigator;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExportFormat {
    Markdown,
    Json,
    Pdf,
}

impl ExportFormat {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Some(Self::Markdown),
            "json" => Some(Self::Json),
            "pdf" => Some(Self::Pdf),
            _ => None,
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Markdown => "text/markdown",
            Self::Json => "application/json",
            Self::Pdf => "application/pdf",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Json => "json",
            Self::Pdf => "pdf",
        }
    }
}

#[derive(Clone)]
pub struct PersonaExportService {
    pool: PgPool,
}

impl PersonaExportService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Export a Persona dossier in the requested format.
    pub async fn export(
        &self,
        persona_id: &str,
        format: ExportFormat,
    ) -> Result<String, ExportError> {
        let investigator = PersonaInvestigator::new(self.pool.clone());
        let dossier = investigator.assemble_dossier(persona_id).await?;
        match format {
            ExportFormat::Json => Ok(serde_json::to_string_pretty(&dossier)?),
            ExportFormat::Markdown => Ok(render_markdown(&dossier)),
            ExportFormat::Pdf => {
                // PDF rendering requires external tooling; return Markdown for now
                Ok(render_markdown(&dossier))
            }
        }
    }
}

fn render_markdown(d: &PersonaDossier) -> String {
    let mut md = String::new();
    md.push_str(&format!("# {}\n\n", d.persona.display_name));
    if let Some(email_address) = &d.persona.email_address {
        md.push_str(&format!("**Email**: {email_address}\n\n"));
    }

    if let Some(role) = &d.persona.tone {
        md.push_str(&format!("**Tone**: {role}\n"));
    }
    if let Some(lang) = &d.persona.language {
        md.push_str(&format!("**Language**: {lang}\n"));
    }
    if let Some(score) = d.persona.trust_score {
        md.push_str(&format!("**Trust**: {score}/100\n"));
    }
    md.push_str(&format!(
        "**Interactions**: {}\n\n",
        d.persona.interaction_count
    ));

    if !d.persona.frequent_topics.is_empty() {
        md.push_str("## Topics\n\n");
        for t in &d.persona.frequent_topics {
            md.push_str(&format!("- {t}\n"));
        }
        md.push('\n');
    }

    if !d.memory_cards.is_empty() {
        md.push_str("## Memory Cards\n\n");
        for card in &d.memory_cards {
            md.push_str(&format!(
                "- **{}**: {} (importance: {})\n",
                card.title, card.description, card.importance
            ));
        }
        md.push('\n');
    }

    if !d.facts.is_empty() {
        md.push_str("## Facts\n\n");
        for fact in &d.facts {
            md.push_str(&format!(
                "- **{}**: {} (source: {}, confidence: {:.0}%)\n",
                fact.fact_type,
                fact.value,
                fact.source,
                fact.confidence * 100.0
            ));
        }
        md.push('\n');
    }

    if !d.timeline.is_empty() {
        md.push_str("## Timeline\n\n");
        for event in &d.timeline {
            md.push_str(&format!(
                "- **{}**: {} ({})\n",
                event.occurred_at.format("%Y-%m-%d"),
                event.title,
                event.event_type
            ));
        }
        md.push('\n');
    }

    if let Some(notes) = &d.persona.notes
        && !notes.is_empty()
    {
        md.push_str(&format!("## Notes\n\n{notes}\n\n"));
    }

    if !d.summary.is_empty() {
        md.push_str(&format!("---\n\n*{summary}*\n", summary = d.summary));
    }

    md
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error(transparent)]
    Investigator(#[from] InvestigatorError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("unsupported export format")]
    UnsupportedFormat,
}
