#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
resource_root="$repo_root/frontend/src-tauri/resources/google-oauth"
target_json="$resource_root/client_secret.json"

source_json="${HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_SOURCE:-${HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH:-}}"

if [ -z "$source_json" ]; then
	cat >&2 <<'EOF'
Unable to prepare bundled Google OAuth Desktop client resource.

Set HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH in docker/.env, or export:
  HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_SOURCE=/absolute/path/to/client_secret.json

The source file must be the downloaded Google OAuth Desktop app JSON with a
top-level "installed" object. The generated bundle resource is copied to
frontend/src-tauri/resources/google-oauth/client_secret.json and ignored by Git.
EOF
	exit 1
fi

if [ ! -f "$source_json" ]; then
	printf '%s\n' "Google OAuth client config file was not found: $source_json" >&2
	exit 1
fi

node - "$source_json" <<'NODE'
const fs = require('fs');

const sourcePath = process.argv[2];
const raw = fs.readFileSync(sourcePath, 'utf8');
const parsed = JSON.parse(raw);
if (!parsed || typeof parsed !== 'object' || !parsed.installed || typeof parsed.installed !== 'object') {
	throw new Error('Google OAuth bundle config must be a Desktop app JSON with top-level "installed".');
}
for (const field of ['client_id', 'auth_uri', 'token_uri']) {
	if (typeof parsed.installed[field] !== 'string' || parsed.installed[field].trim() === '') {
		throw new Error(`Google OAuth Desktop client JSON is missing required field: installed.${field}`);
	}
}
if (parsed.installed.client_secret !== undefined && typeof parsed.installed.client_secret !== 'string') {
	throw new Error('Google OAuth Desktop client JSON field installed.client_secret must be a string when present.');
}
if (parsed.installed.redirect_uris !== undefined && !Array.isArray(parsed.installed.redirect_uris)) {
	throw new Error('Google OAuth Desktop client JSON field installed.redirect_uris must be an array when present.');
}
NODE

mkdir -p "$resource_root"
cp "$source_json" "$target_json"
chmod 0644 "$target_json"

printf '%s\n' "Prepared bundled Google OAuth Desktop client resource: $target_json"
