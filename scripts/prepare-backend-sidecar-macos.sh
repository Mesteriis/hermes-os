#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
binary_root="$repo_root/frontend/src-tauri/binaries"
backend_manifest="$repo_root/backend/Cargo.toml"
backend_bin="hermes-hub-backend"

if [ "$(uname -s)" != "Darwin" ]; then
	printf '%s\n' 'Skipping macOS backend sidecar preparation on non-macOS host.'
	exit 0
fi

case "$(uname -m)" in
	arm64)
		target_triple="${HERMES_MACOS_TARGET_TRIPLE:-aarch64-apple-darwin}"
		;;
	x86_64)
		target_triple="${HERMES_MACOS_TARGET_TRIPLE:-x86_64-apple-darwin}"
		;;
	*)
		printf '%s\n' "Unsupported macOS architecture: $(uname -m)" >&2
		exit 1
		;;
esac

target_root="${CARGO_TARGET_DIR:-$repo_root/target}"
source_bin="$target_root/$target_triple/release/$backend_bin"
target_bin="$binary_root/$backend_bin-$target_triple"

cargo build \
	--manifest-path "$backend_manifest" \
	--bin "$backend_bin" \
	--release \
	--target "$target_triple"

if [ ! -f "$source_bin" ]; then
	printf '%s\n' "Backend sidecar build completed, but $source_bin was not found." >&2
	exit 1
fi

mkdir -p "$binary_root"
cp "$source_bin" "$target_bin"
chmod 0755 "$target_bin"

printf '%s\n' "Prepared bundled backend sidecar: $target_bin"
