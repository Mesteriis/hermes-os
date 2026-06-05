# Knowledge Graph Explorer Design

## Purpose

The next product risk is that Hermes Hub already has a real V2 graph backend, but the `Knowledge Graph` tab is still mostly decorative. This slice turns the tab into a read-only explorer backed by the existing graph APIs.

The goal is to cover three risks together:

- real graph exploration: search nodes, select a node and inspect its neighborhood;
- projection/freshness UX: show whether graph data exists and when it was last projected;
- visual quality: keep the current tab palette and design language, but make the graph feel polished and expensive through restrained motion and clear hierarchy.

## Relevant ADRs

- `ADR-0008 Knowledge Graph First`: graph relationships are durable records with provenance.
- `ADR-0019 Contact Identity Resolution`: fuzzy identity merge is out of scope; exact-email graph edges remain the only automatic identity behavior.
- `ADR-0023 Rebuildable Projections`: graph state is derived and must be rebuildable.
- `ADR-0031 Temporary Desktop Only UI Scope`: no mobile UI design, implementation or validation.
- `ADR-0038`, `ADR-0039`, `ADR-0040`: protected local APIs require local API token and actor identity.
- `ADR-0045 Graph Core Projection`: graph tables are PostgreSQL rebuildable projections; read APIs are local-only and read-only.

## Non-Goals

This slice does not implement:

- graph editing;
- node merge or split;
- fuzzy identity resolution;
- task candidate extraction;
- AI graph summaries;
- a projection rebuild command API;
- pan/zoom engine dependency;
- new graph database, schema or migration;
- mobile graph UI.

## Recommended Approach

Implement an integrated read-only explorer inside the existing `Knowledge Graph` tab.

Use existing frontend API helpers and backend endpoints:

- `GET /api/v2/graph/summary`;
- `GET /api/v2/graph/search?q=<query>&limit=<limit>`;
- `GET /api/v2/graph/neighborhood?node_id=<node_id>&depth=1`.

The frontend should replace the static `graphBuckets` presentation with API-backed state:

- graph summary counts;
- graph search results;
- selected graph node;
- selected node neighborhood;
- evidence rows for returned edges;
- local loading, empty and error states.

Backend changes should be avoided unless the current API shape blocks the frontend. The existing graph API already covers the required read behavior.

## Visual Direction

The current Knowledge Graph colors and visual language must stay intact. Do not introduce a new palette, decorative gradients, loud glow effects or a different brand mood.

The graph should feel premium through restraint:

- subtle node hover and focus transitions;
- a calm selected-node halo using the existing teal/cyan accent;
- edge emphasis only for edges connected to the selected node;
- smooth opacity/transform transitions when search results or neighborhoods change;
- disabled future actions that look intentionally unavailable, not broken;
- no bouncing, particle effects, excessive pulsing or ornamental motion.

Animation should improve comprehension:

- selected node becomes visually anchored;
- newly loaded neighbors fade or slide in lightly;
- loading states use compact skeletons or understated shimmer;
- errors and empty states are static and readable.

Motion must not affect data correctness. It is presentation only.

## UI Structure

### Top Controls

The graph tab keeps the current desktop layout and filter-row style.

Controls:

- graph-specific search input or reuse of the page search state inside the graph tab;
- type chips for node kinds, with only implemented filters enabled;
- freshness/status indicator;
- disabled projection rebuild action with tooltip/title explaining that rebuild command API is not implemented yet.

### Main Canvas

The canvas renders the selected neighborhood:

- selected node in the center;
- direct neighbors arranged deterministically around it;
- edges between returned nodes;
- edge labels or compact relationship chips where space allows;
- node styling by `node_kind`.

The layout should be deterministic and local. No force simulation is required. A simple radial layout is enough for `depth=1` neighborhoods.

If no node is selected:

- show empty guidance if graph summary is empty;
- otherwise show search results and ask the user to select a node.

### Right Rail

The right rail becomes real selected-node context:

- selected node kind, label and stable key;
- generic node metadata from `properties`;
- connected node counts by kind;
- evidence excerpts returned by `GraphNeighborhood.evidence`;
- graph statistics from `GraphSummary`;
- latest projection timestamp.

Static copy such as `Architecture.md`, fake AI summary or hardcoded clusters must be removed from the real graph path.

## Data Flow

On mount:

1. Fetch graph summary.
2. If the graph is empty, show graph empty state.
3. If the graph has nodes, wait for user search or load an initial useful node only if the existing API can produce it without inventing data.

Search:

1. Trim query.
2. Do not call the backend for empty query.
3. Call `searchGraphNodes`.
4. Render results as selectable nodes.
5. Selecting a result calls `fetchGraphNeighborhood`.

Neighborhood:

1. Store the selected node and returned neighborhood.
2. Render `nodes`, `edges` and `evidence`.
3. If the selected node is not found, show a local not-found state and keep summary visible.

Freshness:

- Use `GraphSummary.latest_projection_at`.
- If null and graph is empty, explain that no graph projection has been created.
- If present, render it as the last projection timestamp.

## Error Handling

Errors are scoped to the graph tab:

- summary failure: show graph-level error and retry summary;
- search failure: show search-local error and keep existing selected node if any;
- neighborhood failure: show selected-node detail error and keep search results visible;
- invalid empty search: handled client-side by not calling the API.

The UI must not imply successful writes. Projection rebuild remains disabled until a backend command boundary exists.

## Testing And Validation

Expected validation:

- `cd frontend && pnpm check`;
- `cd frontend && pnpm build`;
- `make validate`;
- `git diff --check`;
- manual local smoke against `http://127.0.0.1:5174` with backend and PostgreSQL running.

Manual smoke criteria:

- Knowledge Graph tab loads without console errors;
- graph summary counts render from API data;
- empty graph state does not show fake nodes;
- searching a known node renders search results;
- selecting a node loads neighborhood and evidence;
- selected-node details come from API data;
- rebuild/projection action is visibly disabled;
- current color palette and visual language remain consistent with the existing Hermes Hub UI.

## Documentation

No new user-facing command is expected. Documentation changes are only required if implementation introduces a new command, env var or operational step.

The implementation plan should explicitly preserve the read-only graph scope and should not add backend write endpoints.
