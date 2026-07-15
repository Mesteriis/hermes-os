use tantivy::Term;
use tantivy::schema::{Field, STORED, STRING, Schema, TEXT, TantivyDocument, Value};

use crate::engines::search::errors::SearchError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchDocument {
    pub object_id: String,
    pub object_kind: String,
    pub title: String,
    pub body: String,
}

impl SearchDocument {
    pub fn validate(&self) -> Result<(), SearchError> {
        validate_non_empty("object_id", &self.object_id)?;
        validate_non_empty("object_kind", &self.object_kind)?;
        validate_non_empty("title", &self.title)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchResult {
    pub object_id: String,
    pub object_kind: String,
    pub title: String,
}

pub struct SearchFields {
    pub schema: Schema,
    pub object_identity: Field,
    pub object_id: Field,
    pub object_kind: Field,
    pub title: Field,
    pub body: Field,
}

impl SearchFields {
    pub fn schema() -> Self {
        let mut schema_builder = Schema::builder();
        let object_identity = schema_builder.add_text_field("object_identity", STRING);
        let object_id = schema_builder.add_text_field("object_id", STRING | STORED);
        let object_kind = schema_builder.add_text_field("object_kind", STRING | STORED);
        let title = schema_builder.add_text_field("title", TEXT | STORED);
        let body = schema_builder.add_text_field("body", TEXT);
        let schema = schema_builder.build();

        Self {
            schema,
            object_identity,
            object_id,
            object_kind,
            title,
            body,
        }
    }
}

pub fn object_identity(document: &SearchDocument) -> String {
    let mut encoded = String::from("search:v1:");
    append_identity_component(&mut encoded, &document.object_kind);
    encoded.push(':');
    append_identity_component(&mut encoded, &document.object_id);
    encoded
}

fn append_identity_component(encoded: &mut String, value: &str) {
    encoded.push_str(&value.len().to_string());
    encoded.push(':');
    encoded.push_str(value);
}

pub fn document_identity_term(field: Field, document: &SearchDocument) -> Term {
    Term::from_field_text(field, &object_identity(document))
}

pub fn required_stored_text(
    document: &TantivyDocument,
    field: Field,
    field_name: &'static str,
) -> Result<String, SearchError> {
    document
        .get_first(field)
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
        .ok_or(SearchError::MissingStoredField(field_name))
}

pub fn validate_non_empty(field_name: &'static str, value: &str) -> Result<(), SearchError> {
    if value.trim().is_empty() {
        return Err(SearchError::EmptyField(field_name));
    }

    Ok(())
}
