use std::path::Path;
use std::sync::Mutex;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, doc};

use crate::engines::search::errors::SearchError;
use crate::engines::search::models::{
    SearchDocument, SearchFields, SearchResult, document_identity_term, object_identity,
    required_stored_text, validate_non_empty,
};

const INDEX_WRITER_MEMORY_BUDGET_BYTES: usize = 50_000_000;

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
