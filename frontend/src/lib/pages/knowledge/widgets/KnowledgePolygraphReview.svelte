<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import * as contradictionsService from '$lib/services/contradictions';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type {
		ContradictionObservation,
		ContradictionReviewState,
		ContradictionSeverity
	} from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		observations: ContradictionObservation[];
		isLoading: boolean;
		error: string;
		reviewingObservationId: string | null;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onReload: () => Promise<void>;
		onReview: (
			observation: ContradictionObservation,
			reviewState: Exclude<ContradictionReviewState, 'suggested'>
		) => Promise<void>;
	}

	let {
		observations,
		isLoading,
		error,
		reviewingObservationId,
		isLayoutEditing,
		isWidgetVisible,
		onReload,
		onReview
	}: Props = $props();

	function severityClass(severity: ContradictionSeverity) {
		return `severity-${contradictionsService.contradictionSeverityTone(severity)}`;
	}
</script>

<section
	class="widget-frame polygraph-review-panel"
	class:editing={isLayoutEditing}
	data-widget-id="knowledge-polygraph-review"
	data-widget-hidden={!isWidgetVisible('knowledge-polygraph-review')}
	aria-busy={isLoading}
>
	<WidgetEditChrome
		widgetId="knowledge-polygraph-review"
		{isLayoutEditing}
		isSelected={false}
		onConfigure={() => {}}
	/>
	<header>
		<div>
			<span class="panel-kicker">{_('Polygraph')}</span>
			<h2>{_('Review Queue')}</h2>
		</div>
		<button type="button" title={_('Reload contradictions')} onclick={() => void onReload()} disabled={isLoading}>
			<Icon icon="tabler:refresh" width="15" height="15" />
		</button>
	</header>

	{#if error}
		<div class="polygraph-state error">
			<span>{error}</span>
			<button type="button" onclick={() => void onReload()}>{_('Retry')}</button>
		</div>
	{:else if isLoading}
		<div class="polygraph-state">
			<span>{_('Loading review items')}</span>
		</div>
	{:else if observations.length === 0}
		<div class="polygraph-state">
			<span>{_('No open contradictions')}</span>
		</div>
	{:else}
		<div class="polygraph-list">
			{#each observations as observation}
				<article class="polygraph-item">
					<div class="polygraph-item-head">
						<strong>{contradictionsService.formatContradictionClaim(observation)}</strong>
						<em class={severityClass(observation.severity)}>{_(observation.severity)}</em>
					</div>
					<p>{contradictionsService.formatContradictionSource(observation.new_source_kind, observation.new_source_id)}</p>
					<small>{contradictionsService.formatContradictionTime(observation.updated_at)}</small>
					<div class="polygraph-actions">
						<button
							type="button"
							disabled={reviewingObservationId === observation.observation_id}
							onclick={() => void onReview(observation, 'user_confirmed')}
						>
							<Icon icon="tabler:check" width="14" height="14" />
							{_('Confirm')}
						</button>
						<button
							type="button"
							disabled={reviewingObservationId === observation.observation_id}
							onclick={() => void onReview(observation, 'user_rejected')}
						>
							<Icon icon="tabler:x" width="14" height="14" />
							{_('Reject')}
						</button>
					</div>
				</article>
			{/each}
		</div>
	{/if}
</section>
