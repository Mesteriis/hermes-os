# Knowledge Graph Explorer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the existing desktop Knowledge Graph tab into a read-only explorer backed by the current V2 graph summary, search and neighborhood APIs.

**Architecture:** Keep this slice frontend-heavy. `frontend/src/lib/api.ts` already exposes graph summary, search and neighborhood helpers, so `frontend/src/routes/+page.svelte` owns explorer state, deterministic radial layout, selected-node details and scoped error states. PostgreSQL graph tables remain rebuildable projections, the UI remains read-only, and protected API calls continue using the local bearer token plus `X-Hermes-Actor-Id`.

**Tech Stack:** SvelteKit 2, Svelte 5 runes, TypeScript, Iconify, existing Hermes Hub CSS, existing Rust/Axum V2 graph APIs, pnpm, Make.

---

## Source Spec

- `docs/superpowers/specs/2026-06-05-graph-explorer-design.md`

## Relevant ADRs

- `docs/adr/ADR-0008-knowledge-graph-first.md` - graph relationships carry provenance and confidence.
- `docs/adr/ADR-0023-rebuildable-projections.md` - graph state is derived and rebuildable.
- `docs/adr/ADR-0031-temporary-desktop-only-ui-scope.md` - no mobile UI design, implementation or validation.
- `docs/adr/ADR-0038-local-event-api-capability-token.md` - protected local APIs use the local bearer token.
- `docs/adr/ADR-0040-local-api-actor-identity.md` - protected local APIs include `X-Hermes-Actor-Id`.
- `docs/adr/ADR-0045-graph-core-projection.md` - V2 graph API is read-only, local-only and backed by PostgreSQL graph projections.

## File Map

- Modify: `frontend/src/routes/+page.svelte` - graph explorer state, layout helpers, Knowledge Graph markup and scoped styling.
- Inspect only: `frontend/src/lib/api.ts` - existing graph API contracts; no expected changes.
- Inspect only: `frontend/package.json` - validation commands.
- No backend files are expected to change for this slice.
- No documentation update is expected unless implementation discovers a new command, env var or operational step.

## Assumptions

Assumption: The existing graph API helpers in `frontend/src/lib/api.ts` are sufficient for this slice.
Reason: `fetchGraphSummary`, `searchGraphNodes` and `fetchGraphNeighborhood` already match the approved spec.
Risk: If implementation discovers an API response shape mismatch, pause and fix the smallest frontend/API contract issue with backend tests before continuing.

Assumption: The current single-file Svelte page remains the local pattern for this UI pass.
Reason: Existing tabs are implemented inside `frontend/src/routes/+page.svelte`, and this slice is intended to replace one tab without broader component restructuring.
Risk: The file stays large; the plan keeps changes grouped and named so a later component split remains straightforward.

Assumption: No frontend unit test framework is configured.
Reason: `frontend/package.json` exposes `check`, `build`, `dev` and `preview`, but no `test` script.
Risk: Behavior is verified with Svelte type checks, production build and manual browser smoke instead of automated UI tests.

---

## Task 1: Add Graph Explorer State And Helpers

**Files:**
- Modify: `frontend/src/routes/+page.svelte`

- [ ] **Step 1: Extend graph API imports**

Replace the current graph-related import list from `$lib/api` at the top of `frontend/src/routes/+page.svelte` with this import block. Keep the existing communication and account setup imports unchanged:

```svelte
	import {
		completeGmailOAuthSetup,
		fetchCommunicationMessage,
		fetchCommunicationMessages,
		fetchGraphNeighborhood,
		fetchGraphSummary,
		fetchV1Status,
		searchGraphNodes,
		setupImapAccount,
		startGmailOAuthSetup,
		type CommunicationMessageDetail,
		type CommunicationMessageDetailItem,
		type CommunicationMessageSummary,
		type GmailOAuthStartResponse,
		type GraphEdge,
		type GraphEvidenceSummary,
		type GraphNeighborhood,
		type GraphNode,
		type GraphNodeKind,
		type GraphRelationshipType,
		type GraphSummary,
		type V1Status
	} from '$lib/api';
```

- [ ] **Step 2: Add graph explorer local types**

Add these type declarations after the existing `Conversation` type:

```svelte
	type GraphCanvasNode = GraphNode & {
		x: number;
		y: number;
		isSelected: boolean;
	};

	type GraphCanvasEdge = GraphEdge & {
		x1: number;
		y1: number;
		x2: number;
		y2: number;
		label: string;
	};

	type GraphPropertyRow = {
		key: string;
		value: string;
	};

	type GraphFilterChip = {
		id: string;
		label: string;
		count: number | null;
		enabled: boolean;
	};
```

- [ ] **Step 3: Add graph state variables**

Replace the current graph state declarations:

```svelte
	let graphSummary = $state<GraphSummary | null>(null);
	let graphError = $state('');
```

with this full graph state block:

```svelte
	let graphSummary = $state<GraphSummary | null>(null);
	let graphError = $state('');
	let isGraphSummaryLoading = $state(false);
	let graphSearchQuery = $state('');
	let graphSearchResults = $state<GraphNode[]>([]);
	let graphSearchError = $state('');
	let isGraphSearchLoading = $state(false);
	let graphSearchSubmitted = $state(false);
	let graphNeighborhood = $state<GraphNeighborhood | null>(null);
	let graphNeighborhoodError = $state('');
	let isGraphNeighborhoodLoading = $state(false);
```

- [ ] **Step 4: Add graph derived values**

Add these derived values after the existing derived values for `selectedAgent`, `activeView` and `activeShortcuts`:

```svelte
	const selectedGraphNode = $derived(graphNeighborhood?.selected_node ?? null);
	const graphCanvasNodes = $derived(buildGraphCanvasNodes(graphNeighborhood));
	const graphCanvasEdges = $derived(buildGraphCanvasEdges(graphNeighborhood, graphCanvasNodes));
	const selectedGraphProperties = $derived(
		selectedGraphNode ? graphPropertyRows(selectedGraphNode.properties) : []
	);
	const graphNeighborCounts = $derived(graphKindCounts(graphNeighborhood?.nodes ?? []));
	const graphFilterChips = $derived(buildGraphFilterChips(graphSummary));
```

- [ ] **Step 5: Replace `loadGraphSummary` with loading-aware behavior**

Replace the current `loadGraphSummary` function with:

```svelte
	async function loadGraphSummary() {
		isGraphSummaryLoading = true;
		try {
			graphSummary = await fetchGraphSummary(apiBaseUrl, apiToken, actorId);
			graphError = '';
		} catch (error) {
			graphError = error instanceof Error ? error.message : 'Unknown graph summary error';
		} finally {
			isGraphSummaryLoading = false;
		}
	}
```

- [ ] **Step 6: Add graph search and neighborhood functions**

Add these functions immediately after `loadGraphSummary`:

```svelte
	async function runGraphSearch() {
		const query = graphSearchQuery.trim();
		graphSearchSubmitted = true;

		if (!query) {
			graphSearchResults = [];
			graphSearchError = '';
			return;
		}

		isGraphSearchLoading = true;
		try {
			graphSearchResults = await searchGraphNodes(apiBaseUrl, apiToken, actorId, query, 20);
			graphSearchError = '';
		} catch (error) {
			graphSearchError = error instanceof Error ? error.message : 'Unknown graph search error';
		} finally {
			isGraphSearchLoading = false;
		}
	}

	async function selectGraphNode(node: GraphNode) {
		graphNeighborhoodError = '';
		isGraphNeighborhoodLoading = true;
		try {
			graphNeighborhood = await fetchGraphNeighborhood(
				apiBaseUrl,
				apiToken,
				actorId,
				node.node_id,
				1
			);
		} catch (error) {
			graphNeighborhoodError =
				error instanceof Error ? error.message : 'Unknown graph neighborhood error';
		} finally {
			isGraphNeighborhoodLoading = false;
		}
	}
```

- [ ] **Step 7: Replace graph summary totals**

Replace the current `graphNodeTotal` and `graphRelationshipTotal` functions with:

```svelte
	function graphNodeTotal() {
		return graphSummary?.node_counts.reduce((total, item) => total + item.count, 0) ?? 0;
	}

	function graphRelationshipTotal() {
		return graphSummary?.edge_counts.reduce((total, item) => total + item.count, 0) ?? 0;
	}
```

- [ ] **Step 8: Add graph rendering helpers**

Add these helpers after `graphNodeKindIcon`:

```svelte
	function graphNodeKindCount(kind: GraphNodeKind) {
		return graphSummary?.node_counts.find((item) => item.key === kind)?.count ?? 0;
	}

	function graphEvidenceTotal() {
		return graphSummary?.evidence_count ?? 0;
	}

	function buildGraphFilterChips(summary: GraphSummary | null): GraphFilterChip[] {
		const nodeKinds: Array<{ id: GraphNodeKind; label: string }> = [
			{ id: 'person', label: 'People' },
			{ id: 'email_address', label: 'Email Addresses' },
			{ id: 'message', label: 'Messages' },
			{ id: 'document', label: 'Documents' }
		];

		return [
			{
				id: 'all',
				label: 'All',
				count: summary?.node_counts.reduce((total, item) => total + item.count, 0) ?? 0,
				enabled: true
			},
			...nodeKinds.map((item) => ({
				id: item.id,
				label: item.label,
				count: summary?.node_counts.find((count) => count.key === item.id)?.count ?? 0,
				enabled: false
			}))
		];
	}

	function buildGraphCanvasNodes(neighborhood: GraphNeighborhood | null): GraphCanvasNode[] {
		if (!neighborhood) {
			return [];
		}

		const selected = neighborhood.selected_node;
		const neighbors = neighborhood.nodes
			.filter((node) => node.node_id !== selected.node_id)
			.slice(0, 14);
		const radius = 38;

		return [
			{ ...selected, x: 50, y: 50, isSelected: true },
			...neighbors.map((node, index) => {
				const angle = (Math.PI * 2 * index) / Math.max(neighbors.length, 1) - Math.PI / 2;
				return {
					...node,
					x: 50 + Math.cos(angle) * radius,
					y: 50 + Math.sin(angle) * radius,
					isSelected: false
				};
			})
		];
	}

	function buildGraphCanvasEdges(
		neighborhood: GraphNeighborhood | null,
		canvasNodes: GraphCanvasNode[]
	): GraphCanvasEdge[] {
		if (!neighborhood) {
			return [];
		}

		const positions = new Map(canvasNodes.map((node) => [node.node_id, node]));
		return neighborhood.edges.flatMap((edge) => {
			const source = positions.get(edge.source_node_id);
			const target = positions.get(edge.target_node_id);
			if (!source || !target) {
				return [];
			}
			return [
				{
					...edge,
					x1: source.x,
					y1: source.y,
					x2: target.x,
					y2: target.y,
					label: formatGraphRelationship(edge.relationship_type)
				}
			];
		});
	}

	function graphKindCounts(nodes: GraphNode[]) {
		const counts = new Map<string, number>();
		for (const node of nodes) {
			counts.set(node.node_kind, (counts.get(node.node_kind) ?? 0) + 1);
		}
		return Array.from(counts.entries())
			.map(([kind, count]) => ({ kind, count }))
			.sort((left, right) => right.count - left.count || left.kind.localeCompare(right.kind));
	}

	function graphPropertyRows(properties: Record<string, unknown>): GraphPropertyRow[] {
		return Object.entries(properties)
			.map(([key, value]) => ({ key, value: formatGraphPropertyValue(value) }))
			.filter((row) => row.value.length > 0)
			.sort((left, right) => left.key.localeCompare(right.key))
			.slice(0, 8);
	}

	function formatGraphPropertyValue(value: unknown): string {
		if (value === null || value === undefined) {
			return '';
		}
		if (Array.isArray(value)) {
			return value.map(formatGraphPropertyValue).filter(Boolean).join(', ');
		}
		if (typeof value === 'object') {
			return JSON.stringify(value);
		}
		return String(value);
	}

	function formatGraphRelationship(type: GraphRelationshipType | string) {
		return type
			.split('_')
			.filter((part) => !['person', 'email', 'address', 'message'].includes(part))
			.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
			.join(' ');
	}

	function formatGraphTimestamp(value: string | null) {
		if (!value) {
			return 'No projection yet';
		}
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) {
			return 'Invalid timestamp';
		}
		return new Intl.DateTimeFormat('en', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		}).format(date);
	}

	function graphEvidenceLabel(evidence: GraphEvidenceSummary) {
		return `${formatGraphKind(evidence.source_kind)} ${evidence.source_id}`;
	}
```

- [ ] **Step 9: Run frontend type check after state/helper edits**

Run:

```sh
cd frontend && pnpm check
```

Expected:

- command exits 0;
- no TypeScript error about missing graph imports, helper names or Svelte rune usage.

- [ ] **Step 10: Commit state and helper changes**

Run:

```sh
git add frontend/src/routes/+page.svelte
git commit -m "feat: add graph explorer state"
```

---

## Task 2: Replace Static Knowledge Graph Markup

**Files:**
- Modify: `frontend/src/routes/+page.svelte`

- [ ] **Step 1: Replace the Knowledge Graph branch**

Replace the full `{:else if currentView === 'knowledge'}` branch in `frontend/src/routes/+page.svelte` with this block:

```svelte
		{:else if currentView === 'knowledge'}
			<section class="knowledge-page">
				<div class="graph-filter-tabs">
					{#each graphFilterChips as item}
						<button
							type="button"
							class:active={item.id === 'all'}
							disabled={!item.enabled}
							title={item.enabled ? `${item.label} graph view` : `${item.label} filtering is not available in this read-only slice`}
						>
							{item.label}
							{#if item.count !== null}<em>{formatNumber(item.count)}</em>{/if}
						</button>
					{/each}
					<button type="button" disabled title="Projection rebuild requires a command API boundary">
						<Icon icon="tabler:refresh" width="15" height="15" />
						Rebuild
					</button>
				</div>

				<div class="knowledge-layout">
					<section class="panel graph-workbench">
						<div class="graph-toolbar">
							<form
								class="graph-search-form"
								onsubmit={(event) => {
									event.preventDefault();
									void runGraphSearch();
								}}
							>
								<Icon icon="tabler:search" width="17" height="17" />
								<input
									bind:value={graphSearchQuery}
									placeholder="Search graph nodes..."
									aria-label="Search graph nodes"
								/>
								<button type="submit" disabled={isGraphSearchLoading || !graphSearchQuery.trim()}>
									{isGraphSearchLoading ? 'Searching' : 'Search'}
								</button>
							</form>
							<button type="button" disabled title="Pan and zoom engine is not part of this slice">
								<Icon icon="tabler:hand-click" width="16" height="16" />
							</button>
							<button type="button" disabled title="Depth is fixed to 1 by the current graph API contract">
								Depth 1
							</button>
						</div>

						<div class="graph-search-strip">
							{#if graphSearchError}
								<div class="graph-strip-message error">
									<span>{graphSearchError}</span>
									<button type="button" onclick={() => void runGraphSearch()}>Retry</button>
								</div>
							{:else if graphSearchResults.length > 0}
								<div class="graph-result-row" aria-label="Graph search results">
									{#each graphSearchResults as node}
										<button
											type="button"
											class:active={selectedGraphNode?.node_id === node.node_id}
											onclick={() => void selectGraphNode(node)}
										>
											<Icon icon={graphNodeKindIcon(node.node_kind)} width="16" height="16" />
											<span>{node.label}</span>
											<em>{formatGraphKind(node.node_kind)}</em>
										</button>
									{/each}
								</div>
							{:else if graphSearchSubmitted && graphSearchQuery.trim()}
								<div class="graph-strip-message">
									<span>No graph nodes found for "{graphSearchQuery.trim()}".</span>
								</div>
							{:else}
								<div class="graph-strip-message">
									<span>Search people, email addresses, messages or documents to load a neighborhood.</span>
								</div>
							{/if}
						</div>

						<div class="knowledge-canvas">
							{#if graphError && !graphSummary}
								<div class="graph-state-card error">
									<Icon icon="tabler:alert-triangle" width="26" height="26" />
									<h2>Graph summary unavailable</h2>
									<p>{graphError}</p>
									<button type="button" onclick={() => void loadGraphSummary()}>Retry summary</button>
								</div>
							{:else if isGraphSummaryLoading && !graphSummary}
								<div class="graph-state-card">
									<Icon icon="tabler:loader-2" width="26" height="26" />
									<h2>Loading graph summary</h2>
									<p>Reading local graph projection metadata.</p>
								</div>
							{:else if graphSummary?.is_empty}
								<div class="graph-state-card">
									<Icon icon="tabler:database-off" width="26" height="26" />
									<h2>No graph projection yet</h2>
									<p>Import contacts, messages or documents, then run the existing projection smoke command to create graph data.</p>
								</div>
							{:else if graphNeighborhood}
								<svg class="graph-edge-layer" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
									{#each graphCanvasEdges as edge}
										<line
											x1={edge.x1}
											y1={edge.y1}
											x2={edge.x2}
											y2={edge.y2}
											class:reviewed={edge.review_state === 'system_accepted' || edge.review_state === 'user_confirmed'}
										/>
									{/each}
								</svg>
								{#each graphCanvasEdges as edge}
									<span
										class="graph-edge-label"
										style={`left:${(edge.x1 + edge.x2) / 2}%; top:${(edge.y1 + edge.y2) / 2}%`}
									>
										{edge.label}
									</span>
								{/each}
								{#each graphCanvasNodes as node}
									<button
										type="button"
										class="graph-node kind-{node.node_kind}"
										class:selected={node.isSelected}
										style={`left:${node.x}%; top:${node.y}%`}
										onclick={() => void selectGraphNode(node)}
										title={`${node.label} - ${formatGraphKind(node.node_kind)}`}
									>
										<Icon icon={graphNodeKindIcon(node.node_kind)} width={node.isSelected ? 28 : 21} height={node.isSelected ? 28 : 21} />
										<strong>{node.label}</strong>
										<span>{formatGraphKind(node.node_kind)}</span>
									</button>
								{/each}
								{#if isGraphNeighborhoodLoading}
									<div class="graph-loading-overlay">
										<Icon icon="tabler:loader-2" width="22" height="22" />
										<span>Loading neighborhood</span>
									</div>
								{/if}
							{:else}
								<div class="graph-state-card">
									<img src="/assets/hermes-logo-mark.png" alt="" />
									<h2>Select a graph node</h2>
									<p>{formatNumber(graphNodeTotal())} nodes and {formatNumber(graphRelationshipTotal())} connections are available from the local projection.</p>
								</div>
							{/if}
						</div>

						<footer class="graph-status-bar">
							<span>Projection: {formatGraphTimestamp(graphSummary?.latest_projection_at ?? null)}</span>
							<span>Evidence: {formatNumber(graphEvidenceTotal())}</span>
							{#if graphNeighborhood?.truncated}<span>Neighborhood truncated at {graphNeighborhood.edge_limit} edges</span>{/if}
							{#if graphNeighborhood?.evidence_truncated}<span>Evidence truncated at {graphNeighborhood.evidence_limit} rows</span>{/if}
						</footer>
					</section>

					<aside class="stacked-rail">
						<section class="panel info-card">
							<h2>Selected Node</h2>
							{#if selectedGraphNode}
								<div class="doc-mini">
									<Icon icon={graphNodeKindIcon(selectedGraphNode.node_kind)} width="24" height="24" />
									<span>
										<strong>{selectedGraphNode.label}</strong>
										<small>{formatGraphKind(selectedGraphNode.node_kind)}</small>
									</span>
								</div>
								<ul class="detail-list node-detail-list">
									<li>Stable key <em>{selectedGraphNode.stable_key}</em></li>
									<li>Created <em>{formatGraphTimestamp(selectedGraphNode.created_at)}</em></li>
									<li>Updated <em>{formatGraphTimestamp(selectedGraphNode.updated_at)}</em></li>
									{#each selectedGraphProperties as row}
										<li>{formatGraphKind(row.key)} <em>{row.value}</em></li>
									{/each}
								</ul>
								{#if graphNeighborhoodError}
									<p class="inline-error">{graphNeighborhoodError}</p>
								{/if}
							{:else}
								<p>Select a search result to inspect graph metadata and evidence.</p>
							{/if}
						</section>

						<section class="panel info-card">
							<h2>Connections</h2>
							{#if graphNeighborCounts.length > 0}
								{#each graphNeighborCounts as item}
									<div class="collection-row">
										<span>{formatGraphKind(item.kind)}</span>
										<em>{item.count}</em>
									</div>
								{/each}
							{:else}
								<p>No selected neighborhood.</p>
							{/if}
						</section>

						<section class="panel info-card">
							<h2>Evidence</h2>
							{#if graphNeighborhood?.evidence.length}
								{#each graphNeighborhood.evidence.slice(0, 5) as evidence}
									<div class="evidence-row">
										<strong>{formatGraphKind(evidence.source_kind)}</strong>
										<p>{evidence.excerpt ?? graphEvidenceLabel(evidence)}</p>
									</div>
								{/each}
							{:else}
								<p>Evidence appears after selecting a node with returned edges.</p>
							{/if}
						</section>

						<section class="panel info-card">
							<h2>Graph Statistics</h2>
							<div class="summary-numbers compact">
								<article><strong>{formatNumber(graphNodeTotal())}</strong><span>Nodes</span></article>
								<article><strong>{formatNumber(graphRelationshipTotal())}</strong><span>Connections</span></article>
								<article><strong>{formatNumber(graphEvidenceTotal())}</strong><span>Evidence</span></article>
								<article><strong>{formatNumber(graphNodeKindCount('person'))}</strong><span>People</span></article>
							</div>
							{#if graphError}<p class="inline-error">{graphError}</p>{/if}
						</section>
					</aside>
				</div>
			</section>
```

- [ ] **Step 2: Remove the static graph bucket data**

Delete the `const graphBuckets = [...]` block from `frontend/src/routes/+page.svelte`.

- [ ] **Step 3: Run frontend type check after markup replacement**

Run:

```sh
cd frontend && pnpm check
```

Expected:

- command exits 0;
- no error about removed `graphBuckets`;
- no error about graph helper names used by markup.

- [ ] **Step 4: Commit markup changes**

Run:

```sh
git add frontend/src/routes/+page.svelte
git commit -m "feat: render graph explorer data"
```

---

## Task 3: Polish Graph Styling Without Changing Visual Language

**Files:**
- Modify: `frontend/src/routes/+page.svelte`

- [ ] **Step 1: Replace graph-specific CSS**

In the `<style>` block, replace the existing Knowledge Graph styles from `.graph-filter-tabs { margin-top: 2px; }` through `.graph-bucket p { ... }` with:

```css
	.graph-filter-tabs {
		margin-top: 2px;
	}

	.graph-filter-tabs em {
		border-radius: 999px;
		background: rgba(142, 174, 174, 0.16);
		color: #d5e7e5;
		font-size: 10px;
		font-style: normal;
		padding: 2px 7px;
	}

	.knowledge-layout {
		grid-template-columns: minmax(760px, 1fr) 310px;
		min-height: 760px;
	}

	.graph-workbench {
		display: grid;
		grid-template-rows: auto auto minmax(0, 1fr) auto;
		overflow: hidden;
	}

	.graph-toolbar {
		display: flex;
		gap: 8px;
		padding: 12px;
	}

	.graph-search-form {
		display: grid;
		grid-template-columns: auto minmax(260px, 1fr) auto;
		gap: 10px;
		align-items: center;
		flex: 1;
		min-height: 38px;
		border: 1px solid rgba(111, 205, 195, 0.14);
		border-radius: 8px;
		background: rgba(4, 21, 24, 0.72);
		padding: 0 8px 0 12px;
		color: #9fb8b6;
	}

	.graph-search-form input {
		width: 100%;
		border: 0;
		outline: 0;
		background: transparent;
		color: #edf8f6;
		font-size: 13px;
	}

	.graph-toolbar button,
	.graph-search-form button,
	.graph-strip-message button,
	.graph-state-card button {
		min-height: 34px;
		border: 1px solid rgba(111, 205, 195, 0.14);
		border-radius: 7px;
		background: rgba(4, 21, 24, 0.72);
		color: #dcefed;
		padding: 0 12px;
		transition:
			border-color 160ms ease,
			background 160ms ease,
			color 160ms ease,
			transform 160ms ease;
	}

	.graph-search-form button:not(:disabled):hover,
	.graph-strip-message button:not(:disabled):hover,
	.graph-state-card button:not(:disabled):hover {
		border-color: rgba(45, 240, 206, 0.38);
		background: rgba(25, 154, 132, 0.18);
		color: #2df0ce;
		transform: translateY(-1px);
	}

	.graph-search-strip {
		min-height: 54px;
		border-top: 1px solid rgba(82, 204, 190, 0.08);
		border-bottom: 1px solid rgba(82, 204, 190, 0.08);
		padding: 9px 12px;
	}

	.graph-result-row {
		display: flex;
		gap: 8px;
		overflow-x: auto;
		padding-bottom: 2px;
	}

	.graph-result-row button {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		flex: 0 0 auto;
		max-width: 240px;
		min-height: 34px;
		border: 1px solid rgba(111, 205, 195, 0.14);
		border-radius: 8px;
		background: rgba(7, 29, 33, 0.76);
		color: #dcefed;
		padding: 0 10px;
		transition:
			border-color 160ms ease,
			background 160ms ease,
			color 160ms ease;
	}

	.graph-result-row button.active,
	.graph-result-row button:hover {
		border-color: rgba(45, 240, 206, 0.42);
		background: rgba(25, 154, 132, 0.2);
		color: #2df0ce;
	}

	.graph-result-row span {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.graph-result-row em {
		color: #8eaead;
		font-size: 10px;
		font-style: normal;
		white-space: nowrap;
	}

	.graph-strip-message {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		min-height: 34px;
		color: #a7bbba;
		font-size: 12px;
	}

	.graph-strip-message.error {
		color: #ffabab;
	}

	.knowledge-canvas {
		position: relative;
		min-height: 610px;
		overflow: hidden;
		background:
			radial-gradient(circle at center, rgba(21, 132, 126, 0.18), transparent 35%),
			linear-gradient(rgba(45, 240, 206, 0.035) 1px, transparent 1px),
			linear-gradient(90deg, rgba(45, 240, 206, 0.025) 1px, transparent 1px);
		background-size: auto, 30px 30px, 30px 30px;
	}

	.graph-edge-layer {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		pointer-events: none;
	}

	.graph-edge-layer line {
		stroke: rgba(45, 240, 206, 0.2);
		stroke-width: 0.14;
		vector-effect: non-scaling-stroke;
		transition: stroke 180ms ease;
	}

	.graph-edge-layer line.reviewed {
		stroke: rgba(45, 240, 206, 0.42);
	}

	.graph-edge-label {
		position: absolute;
		z-index: 2;
		max-width: 120px;
		border: 1px solid rgba(45, 240, 206, 0.12);
		border-radius: 999px;
		background: rgba(5, 22, 25, 0.82);
		color: #8fece1;
		font-size: 10px;
		padding: 3px 7px;
		transform: translate(-50%, -50%);
		white-space: nowrap;
		pointer-events: none;
	}

	.graph-node {
		position: absolute;
		z-index: 3;
		display: grid;
		place-items: center;
		gap: 4px;
		width: 118px;
		min-height: 74px;
		border: 1px solid rgba(45, 240, 206, 0.18);
		border-radius: 8px;
		background: rgba(6, 30, 34, 0.9);
		color: #dcefed;
		padding: 9px;
		transform: translate(-50%, -50%);
		transition:
			border-color 180ms ease,
			background 180ms ease,
			box-shadow 180ms ease,
			transform 180ms ease;
	}

	.graph-node:hover {
		border-color: rgba(45, 240, 206, 0.44);
		background: rgba(8, 44, 48, 0.94);
		transform: translate(-50%, -50%) scale(1.02);
	}

	.graph-node.selected {
		width: 138px;
		min-height: 92px;
		border-color: rgba(45, 240, 206, 0.72);
		background: rgba(7, 50, 51, 0.94);
		box-shadow:
			0 0 0 1px rgba(45, 240, 206, 0.2),
			0 0 34px rgba(45, 240, 206, 0.2);
	}

	.graph-node strong {
		max-width: 100%;
		overflow: hidden;
		text-align: center;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.graph-node span {
		color: #8eaead;
		font-size: 10px;
	}

	.graph-node.kind-person {
		border-color: rgba(43, 235, 175, 0.28);
	}

	.graph-node.kind-email_address,
	.graph-node.kind-message {
		border-color: rgba(44, 174, 255, 0.28);
	}

	.graph-node.kind-document {
		border-color: rgba(142, 98, 255, 0.28);
	}

	.graph-state-card {
		position: absolute;
		top: 50%;
		left: 50%;
		display: grid;
		justify-items: center;
		gap: 10px;
		width: min(420px, 72%);
		border: 1px solid rgba(82, 204, 190, 0.14);
		border-radius: 8px;
		background: rgba(5, 22, 25, 0.78);
		color: #dcefed;
		padding: 28px;
		text-align: center;
		transform: translate(-50%, -50%);
	}

	.graph-state-card.error {
		border-color: rgba(255, 110, 110, 0.3);
		background: rgba(128, 32, 40, 0.22);
	}

	.graph-state-card img {
		width: 56px;
		height: 56px;
		object-fit: contain;
	}

	.graph-state-card h2 {
		margin: 0;
		font-size: 18px;
	}

	.graph-state-card p {
		margin: 0;
		color: #9fb8b6;
		line-height: 1.5;
	}

	.graph-loading-overlay {
		position: absolute;
		right: 18px;
		bottom: 18px;
		display: inline-flex;
		align-items: center;
		gap: 8px;
		border: 1px solid rgba(45, 240, 206, 0.18);
		border-radius: 999px;
		background: rgba(5, 22, 25, 0.86);
		color: #2df0ce;
		padding: 8px 12px;
	}

	.graph-status-bar {
		display: flex;
		gap: 16px;
		align-items: center;
		min-height: 46px;
		border-top: 1px solid rgba(82, 204, 190, 0.08);
		padding: 0 14px;
		color: #a6bbbb;
		font-size: 12px;
	}

	.evidence-row {
		border-bottom: 1px solid rgba(102, 189, 180, 0.08);
		padding: 10px 0;
	}

	.evidence-row strong {
		color: #2df0ce;
		font-size: 12px;
	}

	.evidence-row p {
		margin: 5px 0 0;
		color: #a7bbba;
		line-height: 1.4;
	}
```

- [ ] **Step 2: Keep existing timeline slider CSS intact**

Do not remove the existing `.timeline-slider` styles following the graph block. Other project tabs still use timeline-like UI elements, and the graph status bar does not depend on those rules.

- [ ] **Step 3: Run frontend production build**

Run:

```sh
cd frontend && pnpm build
```

Expected:

- command exits 0;
- Vite build completes without Svelte CSS or accessibility errors.

- [ ] **Step 4: Commit styling changes**

Run:

```sh
git add frontend/src/routes/+page.svelte
git commit -m "style: polish graph explorer"
```

---

## Task 4: Validate Graph Explorer Behavior

**Files:**
- Inspect: `frontend/src/routes/+page.svelte`
- Inspect: running local app at `http://127.0.0.1:5174`

- [ ] **Step 1: Run frontend static validation**

Run:

```sh
cd frontend && pnpm check
cd frontend && pnpm build
```

Expected:

- both commands exit 0;
- no Svelte check failures;
- production build completes.

- [ ] **Step 2: Run repository validation**

Run:

```sh
make validate
```

Expected:

- command exits 0;
- backend validation, frontend check/build and configured smoke gates pass.

- [ ] **Step 3: Run whitespace validation**

Run:

```sh
git diff --check
```

Expected:

- command exits 0;
- no trailing whitespace or conflict markers.

- [ ] **Step 4: Manual browser smoke**

With the existing dev stack running, open `http://127.0.0.1:5174` and verify:

- Knowledge Graph tab loads without console errors.
- Top graph chips show real counts from `GET /api/v2/graph/summary`.
- Empty graph state does not show fake nodes.
- Searching a known graph label calls `GET /api/v2/graph/search`.
- Selecting a search result calls `GET /api/v2/graph/neighborhood`.
- Canvas renders the selected node, returned neighbors and returned edges only.
- Right rail selected-node fields come from the selected API node.
- Evidence panel renders returned evidence excerpts or source identifiers.
- Rebuild, pan/zoom and depth controls are visibly disabled.
- Color palette, tab style, panels and graph background stay consistent with the current Hermes Hub UI.
- Motion is limited to hover, focus, selected-node emphasis and loading overlay transitions.

- [ ] **Step 5: Capture any validation issue before committing**

If any command or manual smoke fails, record the exact failure in the session, fix the smallest relevant code path, rerun the failed validation command, then rerun `git diff --check`.

- [ ] **Step 6: Commit final validation cleanup**

If Task 4 required code edits, run:

```sh
git add frontend/src/routes/+page.svelte
git commit -m "fix: stabilize graph explorer"
```

If Task 4 required no code edits, leave the working tree unchanged.

---

## Task 5: Final Gate And Handoff

**Files:**
- Inspect: full working tree

- [ ] **Step 1: Confirm git state**

Run:

```sh
git status --short
```

Expected:

- empty output if all implementation commits were made;
- only intentionally uncommitted local runtime files if the developer explicitly chose not to commit them.

- [ ] **Step 2: Confirm final validation evidence**

Run:

```sh
cd frontend && pnpm check
cd frontend && pnpm build
make validate
git diff --check
```

Expected:

- all commands exit 0.

- [ ] **Step 3: Final report**

Report:

- changed file: `frontend/src/routes/+page.svelte`;
- the graph explorer is read-only and uses existing V2 graph APIs;
- no backend write endpoint was added;
- no mobile validation was performed due to ADR-0031;
- exact validation commands and outcomes;
- remaining risk: no automated UI test framework is configured, so interaction behavior is covered by manual smoke plus Svelte type/build checks.

---

## Plan Self-Review

- Spec coverage: Tasks 1-3 cover API-backed state, search, selected neighborhood, evidence, freshness, disabled projection controls and restrained visual polish. Task 4 covers manual criteria from the spec. Task 5 covers read-only/no-mobile reporting.
- Red-flag scan: The plan contains concrete file paths, code snippets, commands and expected outputs. It does not rely on undefined helper names.
- Type consistency: Graph helper names used by markup are defined in Task 1. Graph API types match `frontend/src/lib/api.ts`. Svelte event handlers use the existing `onclick`/`onsubmit` style already present in the file.
