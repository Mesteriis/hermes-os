<script lang="ts">
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	interface Props {
		suggestedCandidates: unknown[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { suggestedCandidates, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="tasks-context" data-widget-hidden={!isWidgetVisible('tasks-context')}>
	<WidgetEditChrome widgetId="tasks-context" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card">
		<h2>Recent Candidate Signals</h2>
		{#if suggestedCandidates.length === 0}
			<p class="muted-copy">No pending candidate signals.</p>
		{:else}
			{#each suggestedCandidates.slice(0, 5) as candidate}
				<div class="deadline"><span>{(candidate as Record<string, unknown>).title as string}</span><time>{(candidate as Record<string, unknown>).source_kind as string}</time></div>
			{/each}
		{/if}
	</section>
</div>
