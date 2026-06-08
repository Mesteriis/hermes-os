#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
resource_root="$repo_root/frontend/src-tauri/resources/tdlib"

if [ "$(uname -s)" != "Darwin" ]; then
	printf '%s\n' 'Skipping macOS TDLib resource preparation on non-macOS host.'
	exit 0
fi

case "$(uname -m)" in
	arm64)
		platform_dir="${HERMES_TDLIB_MACOS_PLATFORM_DIR:-macos-arm64}"
		;;
	x86_64)
		platform_dir="${HERMES_TDLIB_MACOS_PLATFORM_DIR:-macos-x64}"
		;;
	*)
		printf '%s\n' "Unsupported macOS architecture: $(uname -m)" >&2
		exit 1
		;;
esac

target_dir="$resource_root/$platform_dir"
target_lib="$target_dir/libtdjson.dylib"

find_source_lib() {
	if [ -n "${HERMES_TDJSON_SOURCE:-}" ]; then
		printf '%s\n' "$HERMES_TDJSON_SOURCE"
		return 0
	fi
	if [ -n "${HERMES_TDJSON_PATH:-}" ]; then
		printf '%s\n' "$HERMES_TDJSON_PATH"
		return 0
	fi
	if command -v brew >/dev/null 2>&1; then
		if brew_prefix="$(brew --prefix tdlib 2>/dev/null)"; then
			printf '%s\n' "$brew_prefix/lib/libtdjson.dylib"
			return 0
		fi
	fi
	if [ -f /opt/homebrew/lib/libtdjson.dylib ]; then
		printf '%s\n' /opt/homebrew/lib/libtdjson.dylib
		return 0
	fi
	if [ -f /usr/local/lib/libtdjson.dylib ]; then
		printf '%s\n' /usr/local/lib/libtdjson.dylib
		return 0
	fi
	return 1
}

build_tdlib_from_source() {
	build_root="${HERMES_TDLIB_BUILD_ROOT:-$repo_root/.local/tdlib-build}"
	source_dir="$build_root/td"
	build_dir="$source_dir/build"
	tdlib_ref="${HERMES_TDLIB_REF:-master}"

	for tool in git cmake; do
		if ! command -v "$tool" >/dev/null 2>&1; then
			printf '%s\n' "TDLib source build requires $tool." >&2
			return 1
		fi
	done

	mkdir -p "$build_root"
	if [ ! -d "$source_dir/.git" ]; then
		git clone https://github.com/tdlib/td.git "$source_dir"
	fi

	git -C "$source_dir" fetch --tags origin
	git -C "$source_dir" checkout "$tdlib_ref"
	cmake -S "$source_dir" -B "$build_dir" -DCMAKE_BUILD_TYPE=Release -DTD_ENABLE_JNI=OFF
	cmake --build "$build_dir" --target tdjson --config Release --parallel "$(sysctl -n hw.ncpu)"

	built_lib="$(find "$build_dir" -type f -name libtdjson.dylib -print -quit)"
	if [ -z "$built_lib" ]; then
		printf '%s\n' "TDLib source build completed, but libtdjson.dylib was not found under $build_dir." >&2
		return 1
	fi

	printf '%s\n' "$built_lib"
}

source_lib="$(find_source_lib || true)"
if [ -z "$source_lib" ] && [ "${HERMES_TDLIB_BUILD_FROM_SOURCE:-0}" = "1" ]; then
	source_lib="$(build_tdlib_from_source || true)"
fi
if [ -z "$source_lib" ] || [ ! -f "$source_lib" ]; then
	cat >&2 <<'EOF'
Unable to find libtdjson.dylib for the macOS bundle.

Provide one of:
  HERMES_TDJSON_SOURCE=/absolute/path/to/libtdjson.dylib
  HERMES_TDJSON_PATH=/absolute/path/to/libtdjson.dylib
  brew install tdlib
  HERMES_TDLIB_BUILD_FROM_SOURCE=1 make tdlib-macos-resource

The generated dylib is copied into frontend/src-tauri/resources/tdlib/<platform>/.
EOF
	exit 1
fi

mkdir -p "$target_dir"
cp "$source_lib" "$target_lib"
chmod 0644 "$target_lib"

printf '%s\n' "Prepared bundled TDLib runtime: $target_lib"
