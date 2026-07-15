NODE ?= node

.PHONY: architecture-check architecture-policy-check cargo-boundaries-check test-architecture validate

architecture-policy-check:
	$(NODE) scripts/check-architecture-policy.mjs

cargo-boundaries-check:
	$(NODE) scripts/check-cargo-boundaries.mjs

test-architecture:
	$(NODE) --test tests/architecture/*.test.mjs

architecture-check: architecture-policy-check cargo-boundaries-check

validate: architecture-check test-architecture
