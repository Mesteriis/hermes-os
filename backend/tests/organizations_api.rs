use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::{build_router, build_router_with_database};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;

const T: &str = "orgs-test-token";

fn cfg() -> AppConfig {
    AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", T)]).expect("cfg")
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-hermes-secret", T)
        .body(Body::empty())
        .expect("req")
}
fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", T)
        .body(Body::from(body.to_string()))
        .expect("req")
}
fn put(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", T)
        .body(Body::from(body.to_string()))
        .expect("req")
}

async fn jb(r: axum::response::Response) -> Value {
    let b = to_bytes(r.into_body(), usize::MAX).await.expect("b");
    serde_json::from_slice(&b).expect("json")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("t")
        .as_nanos()
}

fn enc(v: &str) -> String {
    url::form_urlencoded::byte_serialize(v.as_bytes()).collect()
}

async fn router(db: &str) -> axum::Router {
    let database = Database::connect(Some(db)).await.expect("db");
    build_router_with_database(
        AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", T), ("DATABASE_URL", db)]).expect("cfg"),
        database,
    )
}

// Helper: create org, return org_id
async fn mkorg(app: &axum::Router, s: u128) -> String {
    let r = app
        .clone()
        .oneshot(post(
            "/api/v1/organizations",
            json!({"display_name": format!("T{s}"), "org_type": "technology"}),
        ))
        .await
        .expect("r");
    if r.status().is_success() {
        jb(r).await["organization_id"]
            .as_str()
            .unwrap_or("org:unknown")
            .to_owned()
    } else {
        format!("org:bad:{s}")
    }
}

// ── Auth ───────────────────────────────────────────────────────────────────
#[tokio::test]
async fn orgs_auth_reject() {
    let r = build_router(cfg());
    let resp = r
        .oneshot(
            Request::builder()
                .uri("/api/v1/organizations")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("r");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── CRUD ───────────────────────────────────────────────────────────────────
#[tokio::test]
async fn orgs_crud() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let s = uid();
    let a = router(&db).await;

    // Create
    let r = a
        .clone()
        .oneshot(post(
            "/api/v1/organizations",
            json!({"display_name": format!("A{s}"), "org_type": "technology"}),
        ))
        .await
        .expect("r");
    assert!(r.status().is_success(), "create={}", r.status());
    let oid = jb(r).await["organization_id"].as_str().unwrap().to_owned();

    // Get
    let r = a
        .clone()
        .oneshot(get(&format!("/api/v1/organizations/{}", enc(&oid))))
        .await
        .expect("r");
    assert!(r.status().is_success(), "get={}", r.status());

    // Update
    let r = a
        .clone()
        .oneshot(put(
            &format!("/api/v1/organizations/{}", enc(&oid)),
            json!({"display_name": format!("U{s}")}),
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "update={}", r.status());

    // Archive
    let r = a
        .oneshot(post(
            &format!("/api/v1/organizations/{}/archive", enc(&oid)),
            json!({}),
        ))
        .await
        .expect("r");
    assert!(r.status().is_success(), "archive={}", r.status());
}

#[tokio::test]
async fn orgs_list() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let s = uid();
    let a = router(&db).await;
    mkorg(&a, s).await;
    let r = a.oneshot(get("/api/v1/organizations")).await.expect("r");
    assert!(r.status().is_success(), "list={}", r.status());
}

#[tokio::test]
async fn orgs_search() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let a = router(&db).await;
    let r = a
        .oneshot(get("/api/v1/organizations/search?q=test"))
        .await
        .expect("r");
    assert!(r.status().is_success(), "search={}", r.status());
}

#[tokio::test]
async fn orgs_not_found_404() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let s = uid();
    let a = router(&db).await;
    let r = a
        .oneshot(get(&format!("/api/v1/organizations/org:nx{s}")))
        .await
        .expect("r");
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ── Sub-resource read endpoints ────────────────────────────────────────────
macro_rules! org_test {
    ($name:ident, $path:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
                eprintln!("skip");
                return;
            };
            let s = uid();
            let a = router(&db).await;
            let oid = mkorg(&a, s).await;
            let r = a.oneshot(get(&format!($path, enc(&oid)))).await.expect("r");
            assert!(
                !r.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                r.status()
            );
        }
    };
}

org_test!(orgs_identities, "/api/v1/organizations/{}/identities");
org_test!(orgs_aliases, "/api/v1/organizations/{}/aliases");
org_test!(orgs_domains, "/api/v1/organizations/{}/domains");
org_test!(orgs_departments, "/api/v1/organizations/{}/departments");
org_test!(orgs_contacts, "/api/v1/organizations/{}/contacts");
org_test!(orgs_related, "/api/v1/organizations/{}/related");
org_test!(orgs_timeline, "/api/v1/organizations/{}/timeline");
org_test!(orgs_portals, "/api/v1/organizations/{}/portals");
org_test!(orgs_procedures, "/api/v1/organizations/{}/procedures");
org_test!(orgs_playbooks, "/api/v1/organizations/{}/playbooks");
org_test!(orgs_templates, "/api/v1/organizations/{}/templates");
org_test!(orgs_financial, "/api/v1/organizations/{}/financial");
org_test!(orgs_contracts, "/api/v1/organizations/{}/contracts");
org_test!(orgs_compliance, "/api/v1/organizations/{}/compliance");
org_test!(orgs_services, "/api/v1/organizations/{}/services");
org_test!(orgs_products, "/api/v1/organizations/{}/products");
org_test!(orgs_enrichment, "/api/v1/organizations/{}/enrichment");
org_test!(orgs_risks, "/api/v1/organizations/{}/risks");
org_test!(orgs_health, "/api/v1/organizations/{}/health");
org_test!(orgs_dossier, "/api/v1/organizations/{}/dossier");
org_test!(orgs_brief, "/api/v1/organizations/{}/brief");
org_test!(orgs_context_pack, "/api/v1/organizations/{}/context-pack");

// ── Identity / Alias / Department creation ─────────────────────────────────
macro_rules! org_post_test {
    ($name:ident, $path:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
                eprintln!("skip");
                return;
            };
            let s = uid();
            let a = router(&db).await;
            let oid = mkorg(&a, s).await;
            let r = a
                .oneshot(post(&format!($path, enc(&oid)), $body))
                .await
                .expect("r");
            assert!(
                !r.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                r.status()
            );
        }
    };
}

org_post_test!(
    orgs_post_identity,
    "/api/v1/organizations/{}/identities",
    json!({"identity_type": "email_domain", "identity_value": "ex.com", "source": "manual"})
);
org_post_test!(
    orgs_post_alias,
    "/api/v1/organizations/{}/aliases",
    json!({"name": "AliasCo", "alias_type": "former_name", "source": "manual"})
);
org_post_test!(
    orgs_post_department,
    "/api/v1/organizations/{}/departments",
    json!({"name": "Engineering", "description": "eng"})
);

// ── Watchlist toggle ───────────────────────────────────────────────────────
#[tokio::test]
async fn orgs_watchlist_toggle() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let s = uid();
    let a = router(&db).await;
    let oid = mkorg(&a, s).await;
    let r = a
        .oneshot(post(
            &format!("/api/v1/organizations/{}/watchlist", enc(&oid)),
            json!({}),
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "watchlist={}", r.status());
}
