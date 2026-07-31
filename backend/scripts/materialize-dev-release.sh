#!/usr/bin/env bash

set -euo pipefail
umask 077

backend_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
project_root="$(cd "$backend_root/.." && pwd)"
frontend_root="$project_root/frontend"
local_root="$project_root/.local"
cargo_target_dir="${HERMES_DEV_CARGO_TARGET_DIR:-$backend_root/target}"
release_root="${HERMES_DEV_RELEASE_ROOT:-$local_root/dev-release}"
signing_key="${HERMES_DEV_RELEASE_SIGNING_KEY:-$local_root/dev-release-signing-key.pem}"
tgcalls_root="${HERMES_DEV_TGCALLS_ROOT:-$local_root/dev-native/tgcalls}"
distribution_id="hermes-local-development"
distribution_generation=""
generation_metadata_name="development-distribution-generation"
release_version="1"
build_id="local-development"
target_triple="aarch64-apple-darwin"
staging_root=""

fail() {
	printf 'Hermes development release failed: %s\n' "$1" >&2
	exit 1
}

require_command() {
	command -v "$1" >/dev/null 2>&1 || fail "required command '$1' is unavailable"
}

require_absolute_path() {
	case "$2" in
		/*) ;;
		*) fail "$1 must be an absolute path" ;;
	esac
}

require_regular_file() {
	test -f "$1" && test ! -L "$1" || fail "$2 must be a regular non-symlink file"
}

next_distribution_generation() {
	if ! test -e "$release_root"; then
		printf '%s\n' 1
		return
	fi
	test -d "$release_root" && test ! -L "$release_root" \
		|| fail "existing development release root is invalid"
	metadata_path="$release_root/$generation_metadata_name"
	if ! test -e "$metadata_path"; then
		printf '%s\n' 2
		return
	fi
	require_regular_file "$metadata_path" "development release generation metadata"
	test "$(stat -f '%Lp' "$metadata_path")" = "600" \
		|| fail "development release generation metadata permissions must be 0600"
	installed_generation="$(sed -n '1p' "$metadata_path")"
	test "$(wc -l <"$metadata_path" | tr -d ' ')" = "1" \
		|| fail "development release generation metadata is invalid"
	case "$installed_generation" in
		''|*[!0-9]*) fail "development release generation metadata is invalid" ;;
	esac
	test "$installed_generation" -gt 0 \
		|| fail "development release generation metadata is invalid"
	test "$installed_generation" -lt 9007199254740991 \
		|| fail "development release generation cannot advance"
	printf '%s\n' "$((installed_generation + 1))"
}

remove_staging_root() {
	test -n "$staging_root" || return 0
	case "$staging_root" in
		"$local_root"/dev-release-staging.*)
			rm -rf -- "$staging_root"
			;;
		*)
			fail "refusing to remove an unexpected staging path"
			;;
	esac
	staging_root=""
}

cleanup() {
	status=$?
	trap - EXIT INT TERM HUP
	remove_staging_root
	exit "$status"
}

sha256_file() {
	shasum -a 256 "$1" | awk '{print $1}'
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

for command_name in awk brew cargo git mktemp node pnpm rustc shasum uname; do
	require_command "$command_name"
done
require_absolute_path "HERMES_DEV_CARGO_TARGET_DIR" "$cargo_target_dir"
require_absolute_path "HERMES_DEV_RELEASE_ROOT" "$release_root"
require_absolute_path "HERMES_DEV_RELEASE_SIGNING_KEY" "$signing_key"
require_absolute_path "HERMES_DEV_TGCALLS_ROOT" "$tgcalls_root"
test "$(uname -m)" = "arm64" || fail "the current development release supports macOS arm64 only"

mkdir -p "$local_root"
chmod 700 "$local_root"
distribution_generation="$(next_distribution_generation)"

tdlib_prefix="$(brew --prefix tdlib 2>/dev/null)" \
	|| fail "Homebrew TDLib is required for the Telegram runtime"
tdlib_library_dir="$(cd "$tdlib_prefix/lib" && pwd -P)"
tdjson_candidates=("$tdlib_library_dir"/libtdjson.*.dylib)
test "${#tdjson_candidates[@]}" -eq 1 \
	|| fail "exactly one canonical versioned TDLib dylib is required"
tdjson_path="${tdjson_candidates[0]}"
require_regular_file "$tdjson_path" "TDLib dylib"

tgcalls_path="$tgcalls_root/libhermes_tgcalls_bridge.dylib"
if ! test -f "$tgcalls_path"; then
	printf '%s\n' 'Building the pinned Telegram call bridge for local development...' >&2
	"$backend_root/scripts/build-telegram-tgcalls-bridge-macos.sh" \
		--output-dir "$tgcalls_root" \
		--development-audio-conformance
fi
require_regular_file "$tgcalls_path" "Telegram call bridge"

printf '%s\n' 'Building signed-development runtime and assembly units...' >&2
CARGO_TARGET_DIR="$cargo_target_dir" cargo +1.97.0 build --locked \
	--package hermes-kernel \
	--package hermes-blob-service \
	--package hermes-events-authority-runtime \
	--package hermes-scheduler-runtime \
	--package hermes-storage-runtime \
	--package hermes-telemetry-collector \
	--package hermes-vault-runtime \
	--package hermes-communications-runtime \
	--package hermes-communications-assembly \
	--package hermes-communications-export-runtime \
	--package hermes-communications-export-assembly \
	--package hermes-communication-delivery-intent-runtime \
	--package hermes-communication-delivery-intent-assembly \
	--package hermes-communication-bulk-action-runtime \
	--package hermes-communication-bulk-action-assembly \
	--package hermes-communication-cross-channel-forward-runtime \
	--package hermes-communication-cross-channel-forward-assembly \
	--package hermes-communication-reply-suggestion-runtime \
	--package hermes-communication-reply-suggestion-assembly \
	--package hermes-communication-summary-runtime \
	--package hermes-communication-summary-assembly \
	--package hermes-communication-translation-runtime \
	--package hermes-communication-translation-assembly \
	--package hermes-communication-explanation-runtime \
	--package hermes-communication-explanation-assembly \
	--package hermes-communication-delayed-delivery-runtime \
	--package hermes-communication-delayed-delivery-assembly \
	--package hermes-attachment-security-runtime \
	--package hermes-attachment-security-assembly \
	--package hermes-ollama-ai-runtime \
	--package hermes-ollama-ai-assembly \
	--package hermes-mail-runtime \
	--package hermes-mail-assembly \
	--package hermes-telegram-runtime \
	--package hermes-telegram-assembly \
	--package hermes-whatsapp-runtime \
	--package hermes-whatsapp-assembly \
	--package hermes-zulip-runtime \
	--package hermes-zulip-assembly \
	--package hermes-development-assembly

printf '%s\n' 'Building the Vue browser client for the signed development bundle...' >&2
(
	cd "$frontend_root"
	pnpm build
)

staging_root="$(mktemp -d "$local_root/dev-release-staging.XXXXXX")"
chmod 700 "$staging_root"
scratch_root="$staging_root/scratch"
assembly_root="$staging_root/assemblies"
new_release_root="$staging_root/release"
app_root="$new_release_root/HermesDev.app"
resource_root="$app_root/Contents/Resources/hermes-kernel-release"
mkdir -p \
	"$scratch_root/descriptors" \
	"$assembly_root" \
	"$app_root/Contents/MacOS" \
	"$resource_root"

source_commit="$(git -C "$project_root" rev-parse HEAD)"
lockfile_sha256="$(sha256_file "$backend_root/Cargo.lock")"
sbom_path="$scratch_root/cargo-metadata.json"
toolchain_path="$scratch_root/toolchain.txt"
(
	cd "$backend_root"
	cargo +1.97.0 metadata --locked --format-version 1
) >"$sbom_path"
{
	rustc +1.97.0 -vV
	cargo +1.97.0 -vV
} >"$toolchain_path"
sbom_sha256="$(sha256_file "$sbom_path")"
toolchain_sha256="$(sha256_file "$toolchain_path")"

communications_assembly="$assembly_root/communications"
communications_export_assembly="$assembly_root/communications-export"
communication_delivery_intent_assembly="$assembly_root/communication-delivery-intent"
communication_bulk_action_assembly="$assembly_root/communication-bulk-action"
communication_cross_channel_forward_assembly="$assembly_root/communication-cross-channel-forward"
communication_reply_suggestion_assembly="$assembly_root/communication-reply-suggestion"
communication_summary_assembly="$assembly_root/communication-summary"
communication_translation_assembly="$assembly_root/communication-translation"
communication_explanation_assembly="$assembly_root/communication-explanation"
communication_delayed_delivery_assembly="$assembly_root/communication-delayed-delivery"
attachment_security_assembly="$assembly_root/attachment-security"
ollama_ai_assembly="$assembly_root/ollama-ai"
mail_assembly="$assembly_root/mail"
telegram_assembly="$assembly_root/telegram"
whatsapp_assembly="$assembly_root/whatsapp"
zulip_assembly="$assembly_root/zulip"

"$cargo_target_dir/debug/hermes-communications-assembly" \
	--build-id "$build_id" \
	--output-dir "$communications_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-communications-runtime"
"$cargo_target_dir/debug/hermes-communications-export-assembly" \
	--build-id "$build_id" \
	--output-dir "$communications_export_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-communications-export-runtime"
"$cargo_target_dir/debug/hermes-communication-delivery-intent-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_delivery_intent_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-communication-delivery-intent-runtime"
"$cargo_target_dir/debug/hermes-communication-bulk-action-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_bulk_action_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-communication-bulk-action-runtime"
"$cargo_target_dir/debug/hermes-communication-cross-channel-forward-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_cross_channel_forward_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-communication-cross-channel-forward-runtime"
"$cargo_target_dir/debug/hermes-communication-reply-suggestion-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_reply_suggestion_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-communication-reply-suggestion-runtime"
"$cargo_target_dir/debug/hermes-communication-summary-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_summary_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-communication-summary-runtime"
"$cargo_target_dir/debug/hermes-communication-translation-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_translation_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-communication-translation-runtime"
"$cargo_target_dir/debug/hermes-communication-explanation-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_explanation_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-communication-explanation-runtime"
"$cargo_target_dir/debug/hermes-communication-delayed-delivery-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_delayed_delivery_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-communication-delayed-delivery-runtime"
"$cargo_target_dir/debug/hermes-attachment-security-assembly" \
	--build-id "$build_id" \
	--output-dir "$attachment_security_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-attachment-security-runtime"
"$cargo_target_dir/debug/hermes-ollama-ai-assembly" \
	--build-id "$build_id" \
	--output-dir "$ollama_ai_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-ollama-ai-runtime"
"$cargo_target_dir/debug/hermes-mail-assembly" \
	--build-id "$build_id" \
	--output-dir "$mail_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-mail-runtime"
"$cargo_target_dir/debug/hermes-telegram-assembly" \
	--build-id "$build_id" \
	--output-dir "$telegram_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-telegram-runtime" \
	--tdjson "$tdjson_path" \
	--tgcalls "$tgcalls_path"
"$cargo_target_dir/debug/hermes-whatsapp-assembly" \
	--build-id "$build_id" \
	--output-dir "$whatsapp_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-whatsapp-runtime"
"$cargo_target_dir/debug/hermes-zulip-assembly" \
	--build-id "$build_id" \
	--output-dir "$zulip_assembly" \
	--runtime "$cargo_target_dir/debug/hermes-zulip-runtime"

base_input="$scratch_root/release-input.json"
node "$backend_root/scripts/build-local-platform-release-input.mjs" \
	--target "$target_triple" \
	--artifact-dir "$cargo_target_dir/debug" \
	--browser-bootstrap "$frontend_root/dist/index.html" \
	--browser-assets-dir "$frontend_root/dist/assets" \
	--output "$base_input" \
	--descriptor-dir "$scratch_root/descriptors" \
	--distribution-id "$distribution_id" \
	--generation "$distribution_generation" \
	--release-version "$release_version" \
	--build-id "$build_id" \
	--source-commit "$source_commit" \
	--lockfile-sha256 "$lockfile_sha256" \
	--sbom-sha256 "$sbom_sha256" \
	--toolchain-sha256 "$toolchain_sha256"

if test -e "$signing_key"; then
	require_regular_file "$signing_key" "development release signing key"
	test "$(stat -f '%Lp' "$signing_key")" = "600" \
		|| fail "development release signing key permissions must be 0600"
else
	node "$backend_root/scripts/generate-release-signing-key.mjs" --output "$signing_key"
	chmod 600 "$signing_key"
fi

node "$backend_root/scripts/build-distribution-release.mjs" \
	--input "$base_input" \
	--artifact-fragment "$communications_assembly/communications.release-artifacts.json" \
	--artifact-fragment "$communications_export_assembly/communications_export.release-artifacts.json" \
	--artifact-fragment "$communication_delivery_intent_assembly/communication_delivery_intent.release-artifacts.json" \
	--artifact-fragment "$communication_bulk_action_assembly/communication_bulk_action.release-artifacts.json" \
	--artifact-fragment "$communication_cross_channel_forward_assembly/communication_cross_channel_forward.release-artifacts.json" \
	--artifact-fragment "$communication_reply_suggestion_assembly/communication_reply_suggestion.release-artifacts.json" \
	--artifact-fragment "$communication_summary_assembly/communication_summary.release-artifacts.json" \
	--artifact-fragment "$communication_translation_assembly/communication_translation.release-artifacts.json" \
	--artifact-fragment "$communication_explanation_assembly/communication_explanation.release-artifacts.json" \
	--artifact-fragment "$communication_delayed_delivery_assembly/communication_delayed_delivery.release-artifacts.json" \
	--artifact-fragment "$attachment_security_assembly/attachment-security.release-artifacts.json" \
	--artifact-fragment "$ollama_ai_assembly/ollama-ai.release-artifacts.json" \
	--artifact-fragment "$mail_assembly/mail.release-artifacts.json" \
	--artifact-fragment "$telegram_assembly/telegram.release-artifacts.json" \
	--artifact-fragment "$whatsapp_assembly/whatsapp.release-artifacts.json" \
	--artifact-fragment "$zulip_assembly/zulip.release-artifacts.json" \
	--signing-key "$signing_key" \
	--trust-root "$resource_root/hermes-release-trust-root.pb" \
	--signed-manifest "$resource_root/hermes-signed-distribution-manifest.pb" \
	--distribution-root "$resource_root/distribution"

cp "$cargo_target_dir/debug/hermes-kernel" "$app_root/Contents/MacOS/hermes-kernel"
chmod 700 "$app_root/Contents/MacOS/hermes-kernel"
printf '%s\n' "$distribution_generation" \
	>"$new_release_root/$generation_metadata_name"
chmod 600 "$new_release_root/$generation_metadata_name"

previous_release_root="$local_root/dev-release-previous.$$"
case "$release_root" in
	"$local_root"/*) ;;
	*) fail "development release root must remain inside the project-local state directory" ;;
esac
if test -e "$previous_release_root"; then
	fail "temporary previous release path already exists"
fi
if test -e "$release_root"; then
	mv "$release_root" "$previous_release_root"
fi
mv "$new_release_root" "$release_root"
if test -e "$previous_release_root"; then
	rm -rf -- "$previous_release_root"
fi

kernel_path="$release_root/HermesDev.app/Contents/MacOS/hermes-kernel"
require_regular_file "$kernel_path" "materialized development Kernel"
require_regular_file \
	"$release_root/$generation_metadata_name" \
	"materialized development release generation metadata"
printf 'development-release: ready distribution=%s generation=%s\n' \
	"$distribution_id" "$distribution_generation" >&2
printf '%s\n' "$kernel_path"
