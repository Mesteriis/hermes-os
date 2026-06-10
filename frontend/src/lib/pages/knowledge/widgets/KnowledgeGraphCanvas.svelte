<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { GraphNode, GraphSummary, GraphNeighborhood } from '$lib/api';

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

	interface Props {
		graphCanvasNodes: GraphCanvasNode[];
		graphCanvasEdges: GraphCanvasEdge[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		graphError: string;
		graphSummary: GraphSummary | null;
		isGraphNeighborhoodLoading: boolean;
		isGraphSummaryLoading: boolean;
		graphNeighborhood: GraphNeighborhood | null;

		graphNodeKindIcon: (kind: string) => string;
		formatGraphKind: (kind: string) => string;
		formatNumber: (num: number) => string;
		selectGraphNode: (node: GraphNode) => Promise<void>;
		loadGraphSummary: () => Promise<void>;
		graphNodeTotal: () => number;
		graphRelationshipTotal: () => number;
	}

	let {
		graphCanvasNodes,
		graphCanvasEdges,
		isLayoutEditing,
		isWidgetVisible,
		graphError,
		graphSummary,
		isGraphNeighborhoodLoading,
		isGraphSummaryLoading,
		graphNeighborhood,
		graphNodeKindIcon,
		formatGraphKind,
		formatNumber,
		selectGraphNode,
		loadGraphSummary,
		graphNodeTotal,
		graphRelationshipTotal
	}: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="knowledge-graph-canvas" data-widget-hidden={!isWidgetVisible('knowledge-graph-canvas')}>
	<WidgetEditChrome widgetId="knowledge-graph-canvas" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<div class="knowledge-canvas" aria-busy={isGraphNeighborhoodLoading}>
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
				<p>Import persons, messages or documents, then run the existing projection smoke command to create graph data.</p>
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
				{#each graphCanvasEdges as edge}
					<text
						class="graph-edge-label"
						class:reviewed={edge.review_state === 'system_accepted' || edge.review_state === 'user_confirmed'}
						x={(edge.x1 + edge.x2) / 2}
						y={(edge.y1 + edge.y2) / 2}
					>
						{edge.label}
					</text>
				{/each}
			</svg>
			{#each graphCanvasNodes as node}
				<button
					type="button"
					class="graph-node {node.layoutClass}"
					class:kind-person={node.node_kind === 'person'}
					class:kind-email_address={node.node_kind === 'email_address'}
					class:kind-message={node.node_kind === 'message'}
					class:kind-document={node.node_kind === 'document'}
					class:selected={node.isSelected}
					onclick={() => void selectGraphNode(node as unknown as GraphNode)}
					title={`${node.label} - ${formatGraphKind(node.node_kind)}`}
				>
					<Icon icon={graphNodeKindIcon(node.node_kind)} width={node.isSelected ? 28 : 21} height={node.isSelected ? 28 : 21} />
					<strong>{node.label}</strong>
					<span>{formatGraphKind(node.node_kind)}</span>
				</button>
			{/each}
		{:else}
			<div class="graph-state-card">
				<img src="/assets/hermes-logo-mark.png" alt="" />
				<h2>Select a graph node</h2>
				<p>{formatNumber(graphNodeTotal())} nodes and {formatNumber(graphRelationshipTotal())} connections are available from the local projection. Use Suggested nodes or search to load a neighborhood.</p>
			</div>
		{/if}
		{#if isGraphNeighborhoodLoading}
			<div class="graph-loading-overlay" role="status" aria-live="polite">
				<Icon icon="tabler:loader-2" width="22" height="22" />
				<span>Loading neighborhood</span>
			</div>
		{/if}
	</div>
</div>
