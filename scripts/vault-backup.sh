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
postgres_dump="$backup_dir/postgres.sql"
manifest_path="$backup_dir/manifest.json"
notes_path="$backup_dir/RESTORE.txt"

ensure_dir "$vault_target"

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
  }
}
EOF

cat >"$notes_path" <<EOF
Hermes backup created at $(now_utc)

Contents:
- postgres.sql: logical PostgreSQL dump for $HERMES_POSTGRES_DB
- vault/: host vault data snapshot
- manifest.json: backup metadata

Restore with:
  make vault-restore
EOF

success "Backup created: $backup_dir"
