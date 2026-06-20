# Architecture Communication Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace baseline-based architecture exceptions with a formal communication contract and make code comply.

**Architecture:** The guard reads `scripts/architecture-contract.json`, enforces layer import/cache rules, and rejects baseline or compat exception mechanics. New ADR/documentation make the communication vocabulary canonical and supersede contradictory wording.

**Tech Stack:** Node.js architecture scripts, Rust backend, Vue 3 frontend, Makefile validation.

---

### Task 1: Contract Guard

**Files:**
- Create: `scripts/architecture-contract.json`
- Create: `scripts/check-architecture-contract.test.mjs`
- Modify: `scripts/check-architecture.mjs`
- Modify: `Makefile`
- Delete: `scripts/architecture-boundary-baseline.json`

- [x] **Step 1: Write the failing test**

Run: `node scripts/check-architecture-contract.test.mjs`
Expected: FAIL because `scripts/architecture-contract.json` is missing.

- [ ] **Step 2: Implement contract-driven guard**

Replace baseline loading and stale-baseline logic with contract loading and direct failures for forbidden layer interactions.

- [ ] **Step 3: Run contract and architecture checks**

Run:

```sh
node scripts/check-architecture-contract.test.mjs
node scripts/check-architecture.mjs --self-test
node scripts/check-architecture.mjs
```

Expected: all pass without `scripts/architecture-boundary-baseline.json`.

### Task 2: Architecture Documentation

**Files:**
- Create: `docs/adr/ADR-architecture-communication-contract.md`
- Create: `docs/architecture/component-communication.md`
- Modify: contradictory ADR/docs with Superseded pointers.

- [ ] **Step 1: Add ADR and component communication document**

Document direct calls, command ports, query ports, events, projections and runtime integration APIs.

- [ ] **Step 2: Supersede or clarify older ADRs**

Update ADRs that authorize baseline exceptions, graph-as-domain exception, or channel-as-domain wording.

### Task 3: Code Compliance

**Files:**
- Backend and frontend files reported by the new guard.

- [ ] **Step 1: Fix backend layer violations**

Move integration runtime logic to integrations, move cross-domain orchestration to workflows or command/query ports, and keep platform contracts domain-neutral.

- [ ] **Step 2: Fix frontend layer violations**

Keep product communication state under `frontend/src/domains/communications` and provider runtime/setup UI under `frontend/src/integrations`.

### Task 4: Validation And Wiki

**Files:**
- Modify: `/Users/avm/projects/Personal/infra/mb-avm/docs/08-change-log.md`

- [ ] **Step 1: Run validation**

Run at minimum:

```sh
make architecture-check
make validate
```

- [ ] **Step 2: Update mb-avm wiki**

Record the local Hermes Hub architecture-contract work without secrets.
