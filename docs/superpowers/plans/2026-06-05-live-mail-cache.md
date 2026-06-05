# Live Mail Cache Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist live provider raw mail bytes into local blob storage while keeping PostgreSQL focused on metadata and references.

**Architecture:** Keep the existing provider network clients unchanged: they still return `EmailSyncBatch` with provider raw payloads. Add a second ingestion function in `email_sync.rs` that writes raw Gmail/IMAP bytes through `LocalMailBlobStore`, records blob metadata through `MailStorageStore`, removes raw byte fields from `communication_raw_records.payload`, and stores `raw_blob_id` references instead. The existing `record_email_sync_batch` remains compatible for tests and current callers.

**Tech Stack:** Rust 2024, Tokio fs, SQLx/PostgreSQL, serde_json, base64, existing `mail_storage` module.

---

### Task 1: Disk-backed provider sync import

**Files:**
- Modify: `backend/src/email_sync.rs`
- Modify: `backend/tests/email_provider_network.rs`

- [ ] **Step 1: Write the failing test**

Add a DB-backed test that calls `record_email_sync_batch_with_mail_blobs` with a Gmail payload containing `raw_base64url` and an IMAP payload containing `raw_rfc822_base64`. Assert:

```rust
assert_eq!(report.inserted_or_existing_records, 2);
assert_eq!(report.blobs_upserted, 2);
assert_eq!(raw_payload["raw_base64url"], serde_json::Value::Null);
assert_eq!(raw_payload["raw_rfc822_base64"], serde_json::Value::Null);
assert!(raw_payload["raw_blob_id"].as_str().unwrap().starts_with("blob:v1:sha256:"));
```

- [ ] **Step 2: Run the test to verify RED**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test email_provider_network email_sync_records_provider_batch_with_mail_blobs_against_postgres -- --nocapture
```

Expected: fail with unresolved import/function.

- [ ] **Step 3: Implement minimal code**

Add `EmailSyncBlobImportReport`, `record_email_sync_batch_with_mail_blobs`, raw byte extraction for `raw_rfc822_base64` and `raw_base64url`, local blob write, blob metadata upsert, and payload replacement with `raw_blob_id`, `raw_blob_sha256`, `raw_blob_storage_kind`, `raw_blob_storage_path`, `raw_blob_size_bytes`.

- [ ] **Step 4: Run target tests**

Run:

```sh
cargo test --manifest-path backend/Cargo.toml --test email_provider_network email_sync_records_provider_batch_with_mail_blobs_against_postgres -- --nocapture
```

Expected: pass.

- [ ] **Step 5: Run validation**

Run:

```sh
make backend-validate
git diff --check
```

Expected: pass.

- [ ] **Step 6: Commit**

```sh
git add backend/src/email_sync.rs backend/tests/email_provider_network.rs docs/superpowers/plans/2026-06-05-live-mail-cache.md
git commit -m "feat: persist live mail raw blobs locally"
```
