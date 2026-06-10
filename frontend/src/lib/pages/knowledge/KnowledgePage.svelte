<script lang="ts">
	import Icon from '@iconify/svelte';
	import * as graphService from '$lib/services/graph';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import KnowledgeGraphCanvas from './widgets/KnowledgeGraphCanvas.svelte';
	import KnowledgeNodeInspector from './widgets/KnowledgeNodeInspector.svelte';
	import type {
		GraphNode,
		GraphNodeKind,
		GraphSummary,
		GraphNeighborhood,
		GraphEvidenceSummary,
		GraphEdge,
		GraphRelationshipType
	} from '$lib/api';

	type GraphFilterChip = {
		id: string;
		label: string;
		count: number | null;
		enabled: boolean;
	};

	type GraphCanvasNode = {
		node_id: string;
		node_kind: string;
		label: string;
		x: number;
		y: number;
		isSelected: boolean;
		layoutClass: string;
	};

	type GraphCanvasEdge = {
		x1: number;
		y1: number;
		x2: number;
		y2: number;
		label: string;
		review_state: string;
	};

	type GraphPropertyRow = {
		key: string;
		value: string;
	};

	type NeighborCount = {
		kind: string;
		count: number;
	};

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();

	let graphSummary = $state<GraphSummary | null>(null);
	let graphError = $state('');
	let isGraphSummaryLoading = $state(false);
	let graphNodeChoices = $state<GraphNode[]>([]);
	let graphNodeChoicesError = $state('');
	let isGraphNodeChoicesLoading = $state(false);
	let graphSearchQuery = $state('');
	let graphSearchResults = $state<GraphNode[]>([]);
	let graphSearchError = $state('');
	let isGraphSearchLoading = $state(false);
	let graphSearchSubmitted = $state(false);
	let lastSubmittedGraphSearchQuery = $state('');
	let graphNeighborhood = $state<GraphNeighborhood | null>(null);
	let graphNeighborhoodError = $state('');
	let isGraphNeighborhoodLoading = $state(false);
	let graphNodeChoicesRequestSequence = 0;
	let graphSearchRequestSequence = 0;
	let graphNeighborhoodRequestSequence = 0;

	let selectedGraphNode = $derived(graphNeighborhood?.selected_node ?? null);
	let graphCanvasNodes = $derived(buildGraphCanvasNodes(graphNeighborhood));
	let graphCanvasEdges = $derived(buildGraphCanvasEdges(graphNeighborhood, graphCanvasNodes));
	let selectedGraphProperties = $derived(
		selectedGraphNode ? graphPropertyRows(selectedGraphNode.properties) : []
	);
	let graphNeighborCounts = $derived(graphKindCounts(graphNeighborNodes(graphNeighborhood)));
	let graphFilterChips = $derived(buildGraphFilterChips(graphSummary));

	function formatNumber(value: number) {
		return new Intl.NumberFormat('en-US').format(value);
	}

	function formatGraphKind(kind: GraphNodeKind | string) {
		return kind.split('_').map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join(' ');
	}

	function graphNodeKindIcon(kind: GraphNodeKind | string) {
		switch (kind) {
			case 'person': return 'tabler:user';
			case 'email_address': return 'tabler:mail';
			case 'message': return 'tabler:message';
			case 'document': return 'tabler:file-text';
			case 'project': return 'tabler:cube';
			default: return 'tabler:circle-dot';
		}
	}

	function graphNodeTotal() {
		return graphSummary?.node_counts.reduce((total, item) => total + item.count, 0) ?? 0;
	}

	function graphRelationshipTotal() {
		return graphSummary?.edge_counts.reduce((total, item) => total + item.count, 0) ?? 0;
	}

	function graphEvidenceTotal() {
		return graphSummary?.evidence_count ?? 0;
	}

	function graphNodeKindCount(kind: string) {
		return graphSummary?.node_counts.find((item) => item.key === kind)?.count ?? 0;
	}

	function formatGraphTimestamp(value: string | null) {
		if (!value) return 'No projection yet';
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) return 'Invalid timestamp';
		return new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }).format(date);
	}

	function graphEvidenceLabel(evidence: GraphEvidenceSummary) {
		return `${formatGraphKind(evidence.source_kind)} ${evidence.source_id}`;
	}

	function buildGraphFilterChips(summary: GraphSummary | null): GraphFilterChip[] {
		const nodeKinds: Array<{ id: GraphNodeKind; label: string }> = [
			{ id: 'person', label: 'People' },
			{ id: 'email_address', label: 'Email Addresses' },
			{ id: 'message', label: 'Messages' },
			{ id: 'document', label: 'Documents' },
			{ id: 'project', label: 'Projects' }
		];
		return [
			{ id: 'all', label: 'All', count: summary?.node_counts.reduce((total, item) => total + item.count, 0) ?? 0, enabled: true },
			...nodeKinds.map((item) => ({
				id: item.id, label: item.label,
				count: summary?.node_counts.find((count) => count.key === item.id)?.count ?? 0,
				enabled: false
			}))
		];
	}

	function buildGraphCanvasNodes(neighborhood: GraphNeighborhood | null): GraphCanvasNode[] {
		if (!neighborhood) return [];
		const selected = neighborhood.selected_node;
		const neighbors = neighborhood.nodes.filter((node) => node.node_id !== selected.node_id).slice(0, 14);
		const radius = 38;
		return [
			{ ...selected, x: 50, y: 50, isSelected: true, layoutClass: 'graph-node-position-center' },
			...neighbors.map((node, index) => {
				const angle = (Math.PI * 2 * index) / Math.max(neighbors.length, 1) - Math.PI / 2;
				return { ...node, x: 50 + Math.cos(angle) * radius, y: 50 + Math.sin(angle) * radius, isSelected: false, layoutClass: `graph-node-position-${index}` };
			})
		];
	}

	function buildGraphCanvasEdges(neighborhood: GraphNeighborhood | null, canvasNodes: GraphCanvasNode[]): GraphCanvasEdge[] {
		if (!neighborhood) return [];
		const positions = new Map(canvasNodes.map((node) => [node.node_id, node]));
		return neighborhood.edges.flatMap((edge) => {
			const source = positions.get(edge.source_node_id);
			const target = positions.get(edge.target_node_id);
			if (!source || !target) return [];
			return [{ ...edge, x1: source.x, y1: source.y, x2: target.x, y2: target.y, label: formatGraphRelationship(edge.relationship_type) }];
		});
	}

	function graphNeighborNodes(neighborhood: GraphNeighborhood | null): GraphNode[] {
		if (!neighborhood) return [];
		return neighborhood.nodes.filter((node) => node.node_id !== neighborhood.selected_node.node_id);
	}

	function graphKindCounts(nodes: GraphNode[]): NeighborCount[] {
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
		if (value === null || value === undefined) return '';
		if (Array.isArray(value)) return value.map(formatGraphPropertyValue).filter(Boolean).join(', ');
		if (typeof value === 'object') return JSON.stringify(value);
		return String(value);
	}

	function formatGraphRelationship(type: GraphRelationshipType | string) {
		return type.split('_').filter((part) => !['person', 'email', 'address', 'message'].includes(part))
			.map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join(' ');
	}

	async function loadGraphSummary() {
		isGraphSummaryLoading = true;
		const result = await graphService.loadGraphSummary(0);
		graphSummary = result.summary;
		graphError = result.error;
		isGraphSummaryLoading = false;
	}

	async function loadGraphNodeChoices() {
		const requestSequence = ++graphNodeChoicesRequestSequence;
		isGraphNodeChoicesLoading = true;
		const result = await graphService.loadGraphNodeChoices(requestSequence, graphNodeChoicesRequestSequence);
		graphNodeChoices = result.nodes;
		graphNodeChoicesError = result.error;
		isGraphNodeChoicesLoading = result.isLoading;
		graphNodeChoicesRequestSequence = result.sequence;
	}

	async function runGraphSearch() {
		const requestSequence = ++graphSearchRequestSequence;
		graphSearchSubmitted = true;
		lastSubmittedGraphSearchQuery = graphSearchQuery.trim();
		isGraphSearchLoading = true;
		const result = await graphService.runGraphSearch(graphSearchQuery, requestSequence, graphSearchRequestSequence);
		graphSearchResults = result.results;
		graphSearchError = result.error;
		isGraphSearchLoading = result.isLoading;
	}

	async function selectGraphNode(node: GraphNode) {
		const requestSequence = ++graphNeighborhoodRequestSequence;
		graphNeighborhood = null;
		graphNeighborhoodError = '';
		isGraphNeighborhoodLoading = true;
		const result = await graphService.selectGraphNode(node, requestSequence, graphNeighborhoodRequestSequence);
		graphNeighborhood = result.neighborhood;
		graphNeighborhoodError = result.error;
		isGraphNeighborhoodLoading = result.isLoading;
		graphNeighborhoodRequestSequence = result.sequence;
	}

	$effect(() => {
		loadGraphSummary();
		loadGraphNodeChoices();
	});
</script>

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
			<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="knowledge-toolbar" data-widget-hidden={!isWidgetVisible('knowledge-toolbar')}>
				<WidgetEditChrome widgetId="knowledge-toolbar" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
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
			</div>

			<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="knowledge-search-results" data-widget-hidden={!isWidgetVisible('knowledge-search-results')}>
				<WidgetEditChrome widgetId="knowledge-search-results" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
				<div class="graph-search-strip" aria-live="polite" aria-busy={isGraphSearchLoading || isGraphNodeChoicesLoading}>
					{#if graphSearchError}
						<div class="graph-strip-message error">
							<span>{graphSearchError}</span>
							<button type="button" onclick={() => void runGraphSearch()}>Retry</button>
						</div>
					{:else if graphSearchResults.length > 0}
						<div class="graph-picker">
							<div class="graph-picker-head">
								<span>Search results</span>
								<em>{formatNumber(graphSearchResults.length)}</em>
							</div>
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
						</div>
					{:else if graphSearchSubmitted && lastSubmittedGraphSearchQuery}
						<div class="graph-strip-message">
							<span>No graph nodes found for "{lastSubmittedGraphSearchQuery}".</span>
						</div>
					{:else if graphNodeChoicesError}
						<div class="graph-strip-message error">
							<span>{graphNodeChoicesError}</span>
							<button type="button" onclick={() => void loadGraphNodeChoices()}>Retry</button>
						</div>
					{:else if graphNodeChoices.length > 0}
						<div class="graph-picker">
							<div class="graph-picker-head">
								<span>Suggested nodes</span>
								<em>{formatNumber(graphNodeChoices.length)}</em>
							</div>
							<div class="graph-result-row" aria-label="Suggested graph nodes">
								{#each graphNodeChoices as node}
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
						</div>
					{:else if isGraphNodeChoicesLoading}
						<div class="graph-strip-message">
							<span>Loading selectable graph nodes.</span>
						</div>
					{:else}
						<div class="graph-strip-message">
							<span>No selectable graph nodes returned by the local projection.</span>
						</div>
					{/if}
				</div>
			</div>

			<KnowledgeGraphCanvas
				{graphCanvasNodes}
				{graphCanvasEdges}
				{isLayoutEditing}
				{isWidgetVisible}
				{graphError}
				{graphSummary}
				{isGraphNeighborhoodLoading}
				{isGraphSummaryLoading}
				{graphNeighborhood}
				{graphNodeKindIcon}
				{formatGraphKind}
				{formatNumber}
				{selectGraphNode}
				{loadGraphSummary}
				{graphNodeTotal}
				{graphRelationshipTotal}
			/>

			<footer class="graph-status-bar">
				<span>Projection: {formatGraphTimestamp(graphSummary?.latest_projection_at ?? null)}</span>
				<span>Evidence: {formatNumber(graphEvidenceTotal())}</span>
				{#if graphNeighborhood?.truncated}<span>Neighborhood truncated at {graphNeighborhood.edge_limit} edges</span>{/if}
				{#if graphNeighborhood?.evidence_truncated}<span>Evidence truncated at {graphNeighborhood.evidence_limit} rows</span>{/if}
			</footer>
		</section>

		<KnowledgeNodeInspector
			{selectedGraphNode}
			{selectedGraphProperties}
			{graphNeighborhood}
			{graphNeighborhoodError}
			{graphNeighborCounts}
			{isLayoutEditing}
			{isWidgetVisible}
			{isGraphNeighborhoodLoading}
			{graphError}
			{graphNodeKindIcon}
			{formatGraphKind}
			{formatGraphTimestamp}
			{formatNumber}
			{graphNodeTotal}
			{graphRelationshipTotal}
			{graphEvidenceTotal}
			{graphNodeKindCount}
			{graphEvidenceLabel}
		/>
	</div>

</section>
