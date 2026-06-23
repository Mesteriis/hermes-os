#!/usr/bin/env bash

set -euo pipefail

require_cargo_subcommand() {
	local subcommand="$1"
	local install_hint="$2"

	if cargo "$subcommand" --version >/dev/null 2>&1; then
		return 0
	fi

	echo "Missing cargo subcommand: cargo ${subcommand}" >&2
	echo "Install it with: ${install_hint}" >&2
	exit 1
}

require_binary() {
	local binary="$1"
	local install_hint="$2"

	if command -v "$binary" >/dev/null 2>&1; then
		return 0
	fi

	echo "Missing binary: ${binary}" >&2
	echo "Install it with: ${install_hint}" >&2
	exit 1
}
