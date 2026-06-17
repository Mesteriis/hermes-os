use crate::support::*;
use std::env;

#[tokio::test]
async fn pgvector_semantic_store_indexes_and_searches_sources_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live semantic store test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let store = SemanticEmbeddingStore::new(pool.clone());
    let embedding_model = format!("qwen3-embedding:4b-semantic-{suffix}");

    let extension_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector')")
            .fetch_one(&pool)
            .await
            .expect("vector extension");
    assert!(extension_exists);

    store
        .upsert_embedding(NewSemanticEmbedding {
            source_kind: SemanticSourceKind::Message,
            source_id: &format!("message-semantic-{suffix}"),
            title: "Roadmap planning",
            source_text: "Discussed Hermes Hub AI roadmap and local retrieval.",
            embedding_model: &embedding_model,
            embedding: &unit_embedding(0),
            graph_node_id: Some(&format!("graph:message:{suffix}")),
        })
        .await
        .expect("upsert first embedding");
    store
        .upsert_embedding(NewSemanticEmbedding {
            source_kind: SemanticSourceKind::Document,
            source_id: &format!("document-semantic-{suffix}"),
            title: "Garden notes",
            source_text: "Tomatoes need watering this weekend.",
            embedding_model: &embedding_model,
            embedding: &unit_embedding(8),
            graph_node_id: None,
        })
        .await
        .expect("upsert second embedding");

    let indexed = store
        .upsert_embedding(NewSemanticEmbedding {
            source_kind: SemanticSourceKind::Message,
            source_id: &format!("message-semantic-{suffix}"),
            title: "Roadmap planning",
            source_text: "Discussed Hermes Hub AI roadmap and local retrieval.",
            embedding_model: &embedding_model,
            embedding: &unit_embedding(0),
            graph_node_id: Some(&format!("graph:message:{suffix}")),
        })
        .await
        .expect("idempotent upsert");
    assert_eq!(indexed.source_kind, "message");
    assert_eq!(indexed.embedding_dimension, 2560);

    let results = store
        .search(&embedding_model, &unit_embedding(0), 5)
        .await
        .expect("search");
    assert_eq!(results[0].source_kind, "message");
    assert_eq!(results[0].source_id, format!("message-semantic-{suffix}"));
    assert!(results[0].score > results[1].score);
    assert_eq!(
        results[0].graph_node_id,
        Some(format!("graph:message:{suffix}"))
    );
}
