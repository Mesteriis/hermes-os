#!/usr/bin/env bash

set -euo pipefail

backend_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
project_root="$(cd "$backend_root/.." && pwd)"
frontend_root="$project_root/frontend"
compose_file="$backend_root/development/authenticated/compose.yaml"
legacy_compose_file="$backend_root/development/compose.yaml"
gateway_address="127.0.0.1:9444"
gateway_target="http://$gateway_address"
browser_origin="http://127.0.0.1:5173"
browser_url="$browser_origin/"
data_dir="${HERMES_DEV_DATA_DIR:-$project_root/.local/kernel-dev}"
cargo_target_dir="${HERMES_DEV_CARGO_TARGET_DIR:-$backend_root/target}"
release_root="${HERMES_DEV_RELEASE_ROOT:-$project_root/.local/dev-release}"
distribution_id="hermes-local-development"
generation_metadata_name="development-distribution-generation"
startup_timeout_seconds="${HERMES_DEV_STARTUP_TIMEOUT_SECONDS:-120}"
kernel_pid=""
frontend_pid=""
temporary_dir=""
proof_file=""
compose_started=false

fail() {
	printf 'Hermes development assembly failed: %s\n' "$1" >&2
	exit 1
}

require_command() {
	command -v "$1" >/dev/null 2>&1 || fail "required command '$1' is unavailable"
}

require_absolute_directory_path() {
	case "$2" in
		/*) ;;
		*) fail "$1 must be an absolute path" ;;
	esac
}

require_available_port() {
	if lsof -nP -iTCP:"$1" -sTCP:LISTEN >/dev/null 2>&1; then
		fail "loopback port $1 is already in use"
	fi
}

run_compose() {
	env \
		HERMES_STORAGE_POSTGRES_SECRET_FILE="$data_dir/developer-platform-credentials/postgres-admin-password" \
		HERMES_STORAGE_PGBOUNCER_SECRET_FILE="$data_dir/developer-platform-credentials/pgbouncer-admin-password" \
		HERMES_STORAGE_PGBOUNCER_DATABASES_DIRECTORY="$runtime_dir/storage/pgbouncer" \
		HERMES_STORAGE_PGBOUNCER_AUTH_DIRECTORY="$runtime_dir/storage/pgbouncer/auth" \
		HERMES_STORAGE_PGBOUNCER_RUNTIME_UID="$(id -u)" \
		docker compose -f "$compose_file" "$@"
}

cleanup() {
	status=$?
	trap - EXIT INT TERM HUP
	if test -n "$frontend_pid"; then
		kill -TERM "$frontend_pid" 2>/dev/null || true
	fi
	if test -n "$kernel_pid"; then
		kill -TERM "$kernel_pid" 2>/dev/null || true
	fi
	attempt=0
	while test "$attempt" -lt 50; do
		frontend_alive=false
		kernel_alive=false
		if test -n "$frontend_pid" && kill -0 "$frontend_pid" 2>/dev/null; then
			frontend_alive=true
		fi
		if test -n "$kernel_pid" && kill -0 "$kernel_pid" 2>/dev/null; then
			kernel_alive=true
		fi
		if test "$frontend_alive" = false && test "$kernel_alive" = false; then
			break
		fi
		attempt=$((attempt + 1))
		sleep 0.1
	done
	if test -n "$frontend_pid" && kill -0 "$frontend_pid" 2>/dev/null; then
		kill -KILL "$frontend_pid" 2>/dev/null || true
	fi
	if test -n "$kernel_pid" && kill -0 "$kernel_pid" 2>/dev/null; then
		kill -KILL "$kernel_pid" 2>/dev/null || true
	fi
	if test -n "$frontend_pid"; then
		wait "$frontend_pid" 2>/dev/null || true
	fi
	if test -n "$kernel_pid"; then
		wait "$kernel_pid" 2>/dev/null || true
	fi
	if test -n "$proof_file"; then
		rm -f -- "$proof_file"
	fi
	if test "$compose_started" = true; then
		run_compose down --remove-orphans >/dev/null 2>&1 || true
	fi
	if test -n "$temporary_dir"; then
		rmdir -- "$temporary_dir" 2>/dev/null || true
	fi
	exit "$status"
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

for command_name in cargo curl docker id lsof make mktemp node open pnpm; do
	require_command "$command_name"
done
require_absolute_directory_path "HERMES_DEV_DATA_DIR" "$data_dir"
require_absolute_directory_path "HERMES_DEV_CARGO_TARGET_DIR" "$cargo_target_dir"
require_absolute_directory_path "HERMES_DEV_RELEASE_ROOT" "$release_root"
case "$startup_timeout_seconds" in
	''|*[!0-9]*) fail "HERMES_DEV_STARTUP_TIMEOUT_SECONDS must be a positive integer" ;;
esac
test "$startup_timeout_seconds" -gt 0 || fail "HERMES_DEV_STARTUP_TIMEOUT_SECONDS must be positive"

require_available_port 5173
require_available_port 9444

printf '%s\n' 'Materializing the signed clean-room development release...'
HERMES_DEV_CARGO_TARGET_DIR="$cargo_target_dir" \
	"$backend_root/scripts/materialize-dev-release.sh"
kernel_bin="$release_root/HermesDev.app/Contents/MacOS/hermes-kernel"
generation_metadata="$release_root/$generation_metadata_name"
development_assembly_bin="$cargo_target_dir/debug/hermes-development-assembly"
test -x "$kernel_bin" || fail "signed Kernel development binary is unavailable"
test -x "$development_assembly_bin" || fail "development assembly unit is unavailable"
test -f "$generation_metadata" && test ! -L "$generation_metadata" \
	|| fail "development release generation metadata is unavailable"
test "$(stat -f '%Lp' "$generation_metadata")" = "600" \
	|| fail "development release generation metadata permissions must be 0600"
distribution_generation="$(sed -n '1p' "$generation_metadata")"
test "$(wc -l <"$generation_metadata" | tr -d ' ')" = "1" \
	|| fail "development release generation metadata is invalid"
case "$distribution_generation" in
	''|*[!0-9]*) fail "development release generation metadata is invalid" ;;
esac
test "$distribution_generation" -gt 0 \
	|| fail "development release generation metadata is invalid"

status_output="$("$kernel_bin" --data-dir "$data_dir" status)"
owner_identity="$(printf '%s\n' "$status_output" | sed -n 's/^owner_identity=//p')"
owner_device_signer="$(printf '%s\n' "$status_output" | sed -n 's/^owner_device_signer=//p')"
case "$owner_identity:$owner_device_signer" in
	missing:missing)
		"$kernel_bin" --data-dir "$data_dir" device-key-generate
		"$kernel_bin" --data-dir "$data_dir" initial-owner-enroll \
			--owner-id development-owner \
			--device-id development-desktop
		;;
	missing:ready)
		"$kernel_bin" --data-dir "$data_dir" initial-owner-enroll \
			--owner-id development-owner \
			--device-id development-desktop
		;;
	enrolled:ready) ;;
	enrolled:missing|enrolled:mismatch|enrolled:unavailable)
		fail "the enrolled development owner signer is unavailable or does not match"
		;;
	*)
		fail "development owner identity state is unavailable"
		;;
esac

status_output="$("$kernel_bin" --data-dir "$data_dir" status)"
printf '%s\n' "$status_output" | grep -qx 'owner_identity=enrolled' \
	|| fail "development owner enrollment did not become ready"
printf '%s\n' "$status_output" | grep -qx 'owner_device_signer=ready' \
	|| fail "development owner signer did not become ready"

"$development_assembly_bin" \
	--data-dir "$data_dir" \
	provision-platform
runtime_dir="$("$development_assembly_bin" --data-dir "$data_dir" runtime-directory)"
require_absolute_directory_path "development runtime directory" "$runtime_dir"

printf '%s\n' 'Starting authenticated PostgreSQL, PgBouncer and NATS infrastructure...'
docker compose -f "$legacy_compose_file" down --remove-orphans >/dev/null 2>&1 || true
compose_started=true
run_compose up --detach --wait

temporary_dir="$(mktemp -d "${TMPDIR:-/tmp}/hermes-dev-assembly.XXXXXX")"
chmod 700 "$temporary_dir"
proof_file="$temporary_dir/gateway-proof"
node -e \
	'const fs = require("node:fs"); const crypto = require("node:crypto"); fs.writeFileSync(process.argv[1], crypto.randomBytes(32).toString("hex"), { encoding: "utf8", flag: "wx", mode: 0o600 });' \
	"$proof_file"

printf '%s\n' 'Starting Hermes Kernel and loopback Core Gateway...'
start_kernel() {
	env HERMES_DEVELOPER_VERBOSE=1 "$kernel_bin" \
		--data-dir "$data_dir" \
		serve \
		--browser-gateway-listen-address "$gateway_address" \
		--browser-gateway-origin "$browser_origin" \
		--browser-gateway-rp-id 127.0.0.1 \
		--browser-gateway-development-proxy-proof-file "$proof_file" &
	kernel_pid=$!
}

wait_for_gateway() {
	deadline=$(( $(date +%s) + startup_timeout_seconds ))
	while :; do
		kill -0 "$kernel_pid" 2>/dev/null || fail "Kernel exited before readiness"
		if node "$backend_root/scripts/probe-dev-gateway.mjs" "$proof_file"; then
			break
		fi
		test "$(date +%s)" -lt "$deadline" || fail "Kernel readiness deadline expired"
		sleep 1
	done
}

stop_kernel() {
	kill -TERM "$kernel_pid" 2>/dev/null || true
	wait "$kernel_pid" || true
	kernel_pid=""
}

start_kernel
wait_for_gateway

assembly_status="$(
	"$development_assembly_bin" \
		--data-dir "$data_dir" \
		--distribution-id "$distribution_id" \
		--distribution-generation "$distribution_generation" \
		status
)"
case "$assembly_status" in
	development_assembly=missing)
	printf '%s\n' 'Admitting the exact Communications and provider module plan...'
	;;
	development_assembly=stale)
	printf '%s\n' 'Refreshing the exact Communications and provider module plan...'
	;;
	development_assembly=current) ;;
	*) fail "development assembly state is unavailable" ;;
esac
if test "$assembly_status" != "development_assembly=current"; then
	reconcile_output="$(
		"$development_assembly_bin" \
			--data-dir "$data_dir" \
			--distribution-id "$distribution_id" \
			--distribution-generation "$distribution_generation" \
			admit
	)"
	case "$reconcile_output" in
		development_assembly=admitted|development_assembly=updated) ;;
		*) fail "development assembly reconciliation did not complete" ;;
	esac
	stop_kernel
	start_kernel
	wait_for_gateway
fi

"$development_assembly_bin" \
	--data-dir "$data_dir" \
	--distribution-id "$distribution_id" \
	--distribution-generation "$distribution_generation" \
	start-ensemble

printf '%s\n' 'Starting the Vue/Vite browser client...'
(
	cd "$frontend_root"
	exec env HERMES_DEV_GATEWAY_TARGET="$gateway_target" \
		HERMES_DEV_GATEWAY_PROOF_FILE="$proof_file" \
		pnpm exec vite --host 127.0.0.1 --strictPort
) &
frontend_pid=$!

deadline=$(( $(date +%s) + startup_timeout_seconds ))
while :; do
	kill -0 "$kernel_pid" 2>/dev/null || fail "Kernel exited before browser readiness"
	kill -0 "$frontend_pid" 2>/dev/null || fail "Vite exited before readiness"
	if curl --fail --silent --show-error --max-time 2 "$browser_origin/readyz" >/dev/null; then
		break
	fi
	test "$(date +%s)" -lt "$deadline" || fail "browser readiness deadline expired"
	sleep 1
done

printf 'Hermes development ensemble is ready at %s\n' "$browser_url"
open "$browser_url"
printf '%s\n' 'Browser opened. Press Ctrl-C to stop the full local ensemble.'

while kill -0 "$kernel_pid" 2>/dev/null && kill -0 "$frontend_pid" 2>/dev/null; do
	sleep 1
done
if ! kill -0 "$kernel_pid" 2>/dev/null; then
	wait "$kernel_pid" || child_status=$?
	fail "Kernel stopped unexpectedly with status ${child_status:-0}"
fi
wait "$frontend_pid" || child_status=$?
fail "Vite stopped unexpectedly with status ${child_status:-0}"
