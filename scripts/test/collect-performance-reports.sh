#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

cd "${REPO_ROOT}"

declare -A SUITES=(
	["target/nextest/default/junit.xml"]="default"
	["target/nextest/ci/junit.xml"]="ci"
	["target/nextest/integration/junit.xml"]="integration"
)

found_any=0

for input in "${!SUITES[@]}"; do
	if [[ ! -f "${input}" ]]; then
		continue
	fi

	found_any=1
	node scripts/test/analyze-nextest-junit.mjs \
		--input "${input}" \
		--suite "${SUITES[${input}]}" \
		--output "reports/test-performance/${SUITES[${input}]}"
done

if [[ "${found_any}" -eq 0 ]]; then
	echo "No nextest JUnit XML files found under target/nextest/" >&2
	echo "Run a nextest-based command first, for example: make test-unit" >&2
	exit 1
fi
