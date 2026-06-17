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
ensure_command dropdb
ensure_command createdb
postgres_up

if [ ! -d "$BACKUPS_ROOT" ]; then
	error "No backups directory found at $BACKUPS_ROOT"
	exit 1
fi

backup_dirs=()
while IFS= read -r backup_dir; do
	backup_dirs+=("$backup_dir")
done <<EOF
$(find "$BACKUPS_ROOT" -mindepth 2 -maxdepth 2 -type d | sort)
EOF

if [ "${#backup_dirs[@]}" -eq 0 ]; then
	error "No backups available under $BACKUPS_ROOT"
	exit 1
fi

printf '%s\n' "Available backups:"
select selected_backup in "${backup_dirs[@]}"; do
	if [ -n "${selected_backup:-}" ]; then
		break
	fi
	warn "Invalid selection."
done

postgres_dump="$selected_backup/postgres.sql"
vault_source="$selected_backup/vault"
manifest_path="$selected_backup/manifest.json"

if [ ! -f "$postgres_dump" ] || [ ! -f "$manifest_path" ] || [ ! -d "$vault_source" ]; then
	error "Backup is incomplete: required files are missing in $selected_backup"
	exit 1
fi

confirm_or_exit "Restore will replace database $HERMES_POSTGRES_DB and vault path $HERMES_HOST_VAULT_HOME." "RESTORE"

info "Recreating PostgreSQL database $HERMES_POSTGRES_DB"
PGPASSWORD="$HERMES_POSTGRES_PASSWORD" psql \
	-h 127.0.0.1 \
	-p "$HERMES_POSTGRES_PORT" \
	-U "$HERMES_POSTGRES_USER" \
	-d postgres \
	-v ON_ERROR_STOP=1 \
	-c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '$HERMES_POSTGRES_DB' AND pid <> pg_backend_pid();" >/dev/null
PGPASSWORD="$HERMES_POSTGRES_PASSWORD" dropdb \
	--if-exists \
	-h 127.0.0.1 \
	-p "$HERMES_POSTGRES_PORT" \
	-U "$HERMES_POSTGRES_USER" \
	"$HERMES_POSTGRES_DB"
PGPASSWORD="$HERMES_POSTGRES_PASSWORD" createdb \
	-h 127.0.0.1 \
	-p "$HERMES_POSTGRES_PORT" \
	-U "$HERMES_POSTGRES_USER" \
	"$HERMES_POSTGRES_DB"

info "Restoring PostgreSQL dump"
PGPASSWORD="$HERMES_POSTGRES_PASSWORD" psql \
	-h 127.0.0.1 \
	-p "$HERMES_POSTGRES_PORT" \
	-U "$HERMES_POSTGRES_USER" \
	-d "$HERMES_POSTGRES_DB" \
	-v ON_ERROR_STOP=1 \
	-f "$postgres_dump" >/dev/null

info "Restoring vault data"
rm -rf "$HERMES_HOST_VAULT_HOME"
mkdir -p "$HERMES_HOST_VAULT_HOME"
cp -R "$vault_source"/. "$HERMES_HOST_VAULT_HOME"/

success "Restore completed from $selected_backup"
