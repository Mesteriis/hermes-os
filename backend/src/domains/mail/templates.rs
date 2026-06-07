use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub template_id: String,
    pub name: String,
    pub subject_template: String,
    pub body_template: String,
    pub variables: Vec<String>,
    pub language: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EmailTemplateStore {
    pool: PgPool,
}

impl EmailTemplateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        tpl: &NewEmailTemplate,
    ) -> Result<EmailTemplate, EmailTemplateError> {
        tpl.validate()?;
        let vars: Value = tpl
            .variables
            .iter()
            .map(|v| Value::String(v.clone()))
            .collect();
        let row = sqlx::query(
            r#"INSERT INTO email_templates (template_id, name, subject_template, body_template, variables, language)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (template_id) DO UPDATE SET
                name = EXCLUDED.name, subject_template = EXCLUDED.subject_template,
                body_template = EXCLUDED.body_template, variables = EXCLUDED.variables,
                language = EXCLUDED.language, updated_at = now()
            RETURNING template_id, name, subject_template, body_template, variables, language, created_at, updated_at"#,
        )
        .bind(&tpl.template_id).bind(&tpl.name).bind(&tpl.subject_template).bind(&tpl.body_template)
        .bind(&vars).bind(tpl.language.as_deref())
        .fetch_one(&self.pool).await?;
        row_to_template(row)
    }

    pub async fn list(&self) -> Result<Vec<EmailTemplate>, EmailTemplateError> {
        let rows = sqlx::query(
            r#"SELECT template_id, name, subject_template, body_template, variables, language, created_at, updated_at
            FROM email_templates ORDER BY name"#,
        ).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_template).collect()
    }

    pub async fn get(
        &self,
        template_id: &str,
    ) -> Result<Option<EmailTemplate>, EmailTemplateError> {
        let row = sqlx::query(
            r#"SELECT template_id, name, subject_template, body_template, variables, language, created_at, updated_at
            FROM email_templates WHERE template_id = $1"#,
        ).bind(template_id).fetch_optional(&self.pool).await?;
        row.map(row_to_template).transpose()
    }

    /// Render a template with variables.
    pub fn render(
        &self,
        template: &EmailTemplate,
        vars: &std::collections::HashMap<String, String>,
    ) -> Result<RenderedTemplate, EmailTemplateError> {
        let mut subject = template.subject_template.clone();
        let mut body = template.body_template.clone();
        for (key, value) in vars {
            let placeholder = format!("{{{{{}}}}}", key);
            subject = subject.replace(&placeholder, value);
            body = body.replace(&placeholder, value);
        }
        Ok(RenderedTemplate { subject, body })
    }
}

#[derive(Clone, Debug)]
pub struct NewEmailTemplate {
    pub template_id: String,
    pub name: String,
    pub subject_template: String,
    pub body_template: String,
    pub variables: Vec<String>,
    pub language: Option<String>,
}

impl NewEmailTemplate {
    fn validate(&self) -> Result<(), EmailTemplateError> {
        if self.template_id.trim().is_empty() {
            return Err(EmailTemplateError::InvalidTemplate("template_id empty"));
        }
        if self.name.trim().is_empty() {
            return Err(EmailTemplateError::InvalidTemplate("name empty"));
        }
        if self.subject_template.trim().is_empty() {
            return Err(EmailTemplateError::InvalidTemplate(
                "subject_template empty",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RenderedTemplate {
    pub subject: String,
    pub body: String,
}

fn row_to_template(row: PgRow) -> Result<EmailTemplate, EmailTemplateError> {
    let vars: Value = row.try_get("variables")?;
    let variables: Vec<String> = vars
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    Ok(EmailTemplate {
        template_id: row.try_get("template_id")?,
        name: row.try_get("name")?,
        subject_template: row.try_get("subject_template")?,
        body_template: row.try_get("body_template")?,
        variables,
        language: row.try_get("language")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Debug, Error)]
pub enum EmailTemplateError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("invalid template: {0}")]
    InvalidTemplate(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_template_substitutes_variables() {
        let tpl = EmailTemplate {
            template_id: "t1".into(),
            name: "Test".into(),
            subject_template: "Hello {{name}}".into(),
            body_template: "Hi {{name}},\n\n{{message}}".into(),
            variables: vec!["name".into(), "message".into()],
            language: Some("en".into()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let mut vars: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        vars.insert("name".into(), "Alice".into());
        vars.insert("message".into(), "How are you?".into());
        let mut subject = tpl.subject_template.clone();
        let mut body = tpl.body_template.clone();
        for (key, value) in &vars {
            subject = subject.replace(&format!("{{{{{}}}}}", key), value);
            body = body.replace(&format!("{{{{{}}}}}", key), value);
        }
        assert_eq!(subject, "Hello Alice");
        assert_eq!(body, "Hi Alice,\n\nHow are you?");
    }
}
