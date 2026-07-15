#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=./lib/env.sh
source "$SCRIPT_DIR/lib/env.sh"
# shellcheck source=./lib/postgres.sh
source "$SCRIPT_DIR/lib/postgres.sh"

load_hermes_env
ensure_postgres_client_dependencies
ensure_command pg_dump
postgres_up

backup_day="$(today_utc)"
backup_stamp="$(timestamp_compact_utc)"
backup_dir="$BACKUPS_ROOT/$backup_day/$backup_stamp"
vault_target="$backup_dir/vault"
mail_blobs_target="$backup_dir/mail-blobs"
mail_blobs_integrity_path="$backup_dir/mail-blobs.sha256"
mail_blobs_stats_path="$backup_dir/.mail-blobs-stats.$$"
postgres_dump="$backup_dir/postgres.sql"
manifest_path="$backup_dir/manifest.json"
notes_path="$backup_dir/RESTORE.txt"

ensure_dir "$vault_target"
ensure_dir "$mail_blobs_target"

info "Creating PostgreSQL dump"
PGPASSWORD="$HERMES_POSTGRES_PASSWORD" pg_dump \
	--host 127.0.0.1 \
	--port "$HERMES_POSTGRES_PORT" \
	--username "$HERMES_POSTGRES_USER" \
	--dbname "$HERMES_POSTGRES_DB" \
	--no-owner \
	--no-privileges \
	--file "$postgres_dump"

vault_present=false
if [ -d "$HERMES_HOST_VAULT_HOME" ]; then
	vault_present=true
	info "Copying vault data from $HERMES_HOST_VAULT_HOME"
	cp -R "$HERMES_HOST_VAULT_HOME"/. "$vault_target"/
else
	warn "Vault directory does not exist yet: $HERMES_HOST_VAULT_HOME"
fi

# PostgreSQL stores only metadata and references. Persist the content-addressed
# blob tree separately so a restored message never points at a missing payload.
ensure_dir "$MAIL_BLOB_ROOT"
ensure_regular_file_tree "$MAIL_BLOB_ROOT" "Mail blob storage $MAIL_BLOB_ROOT"
info "Copying mail blob storage from $MAIL_BLOB_ROOT"
cp -R "$MAIL_BLOB_ROOT"/. "$mail_blobs_target"/
write_directory_integrity_manifest "$mail_blobs_target" "$mail_blobs_integrity_path" >"$mail_blobs_stats_path"
IFS=$'\t' read -r mail_blob_file_count mail_blob_total_bytes <"$mail_blobs_stats_path"
rm -f "$mail_blobs_stats_path"
if ! [[ "$mail_blob_file_count" =~ ^[0-9]+$ ]] || ! [[ "$mail_blob_total_bytes" =~ ^[0-9]+$ ]]; then
	error "Mail blob backup inventory did not return valid totals."
	exit 1
fi

git_revision="unknown"
if git -C "$REPO_ROOT" rev-parse --short HEAD >/dev/null 2>&1; then
	git_revision="$(git -C "$REPO_ROOT" rev-parse --short HEAD)"
fi

cat >"$manifest_path" <<EOF
{
  "created_at": "$(now_utc)",
  "backup_dir": "$(json_escape "$backup_dir")",
  "git_revision": "$(json_escape "$git_revision")",
  "database": {
    "name": "$(json_escape "$HERMES_POSTGRES_DB")",
    "user": "$(json_escape "$HERMES_POSTGRES_USER")",
    "host": "127.0.0.1",
    "port": $HERMES_POSTGRES_PORT,
    "dump_file": "postgres.sql"
  },
  "vault": {
    "source_path": "$(json_escape "$HERMES_HOST_VAULT_HOME")",
    "relative_path": "vault",
    "present": $vault_present
  },
  "mail_blobs": {
    "source_path": "$(json_escape "$MAIL_BLOB_ROOT")",
    "relative_path": "mail-blobs",
    "integrity_file": "mail-blobs.sha256",
    "file_count": $mail_blob_file_count,
    "total_bytes": $mail_blob_total_bytes
  }
}
EOF

cat >"$notes_path" <<EOF
Hermes backup created at $(now_utc)

Contents:
- postgres.sql: logical PostgreSQL dump for $HERMES_POSTGRES_DB
- vault/: host vault data snapshot
- mail-blobs/: content-addressed mail message and attachment blobs
- mail-blobs.sha256: SHA-256 and byte-size inventory for mail blobs
- manifest.json: backup metadata

Restore with:
  make vault-restore
EOF

success "Backup created: $backup_dir"
