use std::path::Path;
use std::sync::Mutex;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, STORED, STRING, Schema, TEXT, TantivyDocument, Value};
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, Term, doc};
use thiserror::Error;

const INDEX_WRITER_MEMORY_BUDGET_BYTES: usize = 50_000_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchDocument {
    pub object_id: String,
    pub object_kind: String,
    pub title: String,
    pub body: String,
}

impl SearchDocument {
    fn validate(&self) -> Result<(), SearchError> {
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

pub struct SearchIndex {
    index: Index,
    reader: IndexReader,
    writer: Mutex<IndexWriter>,
    fields: SearchFields,
}

impl SearchIndex {
    pub fn open_or_create(path: &Path) -> Result<Self, SearchError> {
        let fields = SearchFields::schema();
        let index = Index::open_or_create(
            tantivy::directory::MmapDirectory::open(path)?,
            fields.schema.clone(),
        )?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;
        let writer = Mutex::new(index.writer(INDEX_WRITER_MEMORY_BUDGET_BYTES)?);

        Ok(Self {
            index,
            reader,
            writer,
            fields,
        })
    }

    pub fn upsert_document(&self, document: &SearchDocument) -> Result<(), SearchError> {
        document.validate()?;

        let writer = self
            .writer
            .lock()
            .map_err(|_| SearchError::WriterLockPoisoned)?;
        writer.delete_term(document_identity_term(
            self.fields.object_identity,
            document,
        ));
        writer.add_document(doc!(
            self.fields.object_identity => object_identity(document),
            self.fields.object_id => document.object_id.clone(),
            self.fields.object_kind => document.object_kind.clone(),
            self.fields.title => document.title.clone(),
            self.fields.body => document.body.clone(),
        ))?;

        Ok(())
    }

    pub fn commit(&self) -> Result<(), SearchError> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| SearchError::WriterLockPoisoned)?;
        writer.commit()?;
        self.reader.reload()?;

        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        validate_non_empty("query", query)?;
        if limit == 0 {
            return Err(SearchError::InvalidLimit);
        }

        let query_parser =
            QueryParser::for_index(&self.index, vec![self.fields.title, self.fields.body]);
        let query = query_parser.parse_query(query.trim())?;
        let searcher = self.reader.searcher();
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        top_docs
            .into_iter()
            .map(|(_score, doc_address)| {
                let document = searcher.doc::<TantivyDocument>(doc_address)?;
                Ok(SearchResult {
                    object_id: required_stored_text(&document, self.fields.object_id, "object_id")?,
                    object_kind: required_stored_text(
                        &document,
                        self.fields.object_kind,
                        "object_kind",
                    )?,
                    title: required_stored_text(&document, self.fields.title, "title")?,
                })
            })
            .collect()
    }
}

struct SearchFields {
    schema: Schema,
    object_identity: Field,
    object_id: Field,
    object_kind: Field,
    title: Field,
    body: Field,
}

impl SearchFields {
    fn schema() -> Self {
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

fn object_identity(document: &SearchDocument) -> String {
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

fn document_identity_term(field: Field, document: &SearchDocument) -> Term {
    Term::from_field_text(field, &object_identity(document))
}

fn required_stored_text(
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

fn validate_non_empty(field_name: &'static str, value: &str) -> Result<(), SearchError> {
    if value.trim().is_empty() {
        return Err(SearchError::EmptyField(field_name));
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error(transparent)]
    Tantivy(#[from] tantivy::TantivyError),

    #[error(transparent)]
    OpenDirectory(#[from] tantivy::directory::error::OpenDirectoryError),

    #[error(transparent)]
    QueryParser(#[from] tantivy::query::QueryParserError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("search limit must be greater than zero")]
    InvalidLimit,

    #[error("search index writer lock was poisoned")]
    WriterLockPoisoned,

    #[error("search result missing stored field: {0}")]
    MissingStoredField(&'static str),
}
