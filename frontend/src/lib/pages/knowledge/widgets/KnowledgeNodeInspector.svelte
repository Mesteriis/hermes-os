<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { GraphNode, GraphNeighborhood, GraphEvidenceSummary } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	type GraphPropertyRow = {
		key: string;
		value: string;
	};

	type NeighborCount = {
		kind: string;
		count: number;
	};

	interface Props {
		selectedGraphNode: GraphNode | null;
		selectedGraphProperties: GraphPropertyRow[];
		graphNeighborhood: GraphNeighborhood | null;
		graphNeighborhoodError: string;
		graphNeighborCounts: NeighborCount[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		isGraphNeighborhoodLoading: boolean;
		graphError: string;

		graphNodeKindIcon: (kind: string) => string;
		formatGraphKind: (kind: string) => string;
		formatGraphTimestamp: (ts: string | null) => string;
		formatNumber: (num: number) => string;
		graphNodeTotal: () => number;
		graphRelationshipTotal: () => number;
		graphEvidenceTotal: () => number;
		graphNodeKindCount: (kind: string) => number;
		graphEvidenceLabel: (evidence: GraphEvidenceSummary) => string;
	}

	let {
		selectedGraphNode,
		selectedGraphProperties,
		graphNeighborhood,
		graphNeighborhoodError,
		graphNeighborCounts,
		isLayoutEditing,
		isWidgetVisible,
		isGraphNeighborhoodLoading,
		graphError,
		graphNodeKindIcon,
		formatGraphKind,
		formatGraphTimestamp,
		formatNumber,
		graphNodeTotal,
		graphRelationshipTotal,
		graphEvidenceTotal,
		graphNodeKindCount,
		graphEvidenceLabel
	}: Props = $props();
</script>

<aside class="stacked-rail">
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="knowledge-node-inspector" data-widget-hidden={!isWidgetVisible('knowledge-node-inspector')}>
		<WidgetEditChrome widgetId="knowledge-node-inspector" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card">
			<h2>{_('Selected Node')}</h2>
			{#if selectedGraphNode}
				<div class="doc-mini">
					<Icon icon={graphNodeKindIcon(selectedGraphNode.node_kind)} width="24" height="24" />
					<span>
						<strong>{selectedGraphNode.label}</strong>
						<small>{formatGraphKind(selectedGraphNode.node_kind)}</small>
					</span>
				</div>
				<ul class="detail-list node-detail-list">
					<li>{_('Stable key')} <em>{selectedGraphNode.stable_key}</em></li>
					<li>{_('Created')} <em>{formatGraphTimestamp(selectedGraphNode.created_at)}</em></li>
					<li>{_('Updated')} <em>{formatGraphTimestamp(selectedGraphNode.updated_at)}</em></li>
					{#each selectedGraphProperties as row}
						<li>{formatGraphKind(row.key)} <em>{row.value}</em></li>
					{/each}
				</ul>
			{:else}
				<p>{_('Select a graph node to inspect metadata and evidence.')}</p>
			{/if}
			{#if graphNeighborhoodError}
				<p class="inline-error" role="status" aria-live="polite">{graphNeighborhoodError}</p>
			{/if}
		</section>
	</div>

	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="knowledge-connections" data-widget-hidden={!isWidgetVisible('knowledge-connections')}>
		<WidgetEditChrome widgetId="knowledge-connections" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card">
			<h2>{_('Connections')}</h2>
			{#if graphNeighborCounts.length > 0}
				{#each graphNeighborCounts as item}
					<div class="collection-row">
						<span>{formatGraphKind(item.kind)}</span>
						<em>{item.count}</em>
					</div>
				{/each}
			{:else}
				<p>{_('No returned connections.')}</p>
			{/if}
		</section>
	</div>

	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="knowledge-evidence-context" data-widget-hidden={!isWidgetVisible('knowledge-evidence-context')}>
		<WidgetEditChrome widgetId="knowledge-evidence-context" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card">
			<h2>{_('Evidence')}</h2>
			{#if graphNeighborhood?.evidence.length}
				{#each graphNeighborhood.evidence.slice(0, 5) as evidence}
					<div class="evidence-row">
						<strong>{formatGraphKind(evidence.source_kind)}</strong>
						<p>{evidence.excerpt ?? graphEvidenceLabel(evidence)}</p>
					</div>
				{/each}
			{:else}
				<p>{_('Evidence appears after selecting a node with returned edges.')}</p>
			{/if}
		</section>
	</div>

	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="knowledge-graph-summary" data-widget-hidden={!isWidgetVisible('knowledge-graph-summary')}>
		<WidgetEditChrome widgetId="knowledge-graph-summary" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card">
			<h2>{_('Graph Statistics')}</h2>
			<div class="summary-numbers compact">
				<article><strong>{formatNumber(graphNodeTotal())}</strong><span>{_('Nodes')}</span></article>
				<article><strong>{formatNumber(graphRelationshipTotal())}</strong><span>{_('Connections')}</span></article>
				<article><strong>{formatNumber(graphEvidenceTotal())}</strong><span>{_('Evidence')}</span></article>
				<article><strong>{formatNumber(graphNodeKindCount('person'))}</strong><span>{_('People')}</span></article>
			</div>
			{#if graphError}<p class="inline-error">{graphError}</p>{/if}
		</section>
	</div>
</aside>
