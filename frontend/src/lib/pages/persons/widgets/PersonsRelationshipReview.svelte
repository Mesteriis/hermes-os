<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import * as relationshipsService from '$lib/services/relationships';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { Relationship, RelationshipReviewState } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		relationships: Relationship[];
		selectedPersonaId: string | null;
		isLoading: boolean;
		error: string;
		reviewingRelationshipId: string | null;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onReload: () => Promise<void>;
		onReview: (
			relationship: Relationship,
			reviewState: Exclude<RelationshipReviewState, 'suggested' | 'system_accepted'>
		) => Promise<void>;
	}

	let {
		relationships,
		selectedPersonaId,
		isLoading,
		error,
		reviewingRelationshipId,
		isLayoutEditing,
		isWidgetVisible,
		onReload,
		onReview
	}: Props = $props();

	let suggestedRelationships = $derived(
		relationships.filter((relationship) => relationship.review_state === 'suggested')
	);

	function relationshipPeer(relationship: Relationship) {
		if (selectedPersonaId && relationship.source_entity_id === selectedPersonaId) {
			return relationshipsService.formatRelationshipEndpoint(
				relationship.target_entity_kind,
				relationship.target_entity_id
			);
		}
		if (selectedPersonaId && relationship.target_entity_id === selectedPersonaId) {
			return relationshipsService.formatRelationshipEndpoint(
				relationship.source_entity_kind,
				relationship.source_entity_id
			);
		}
		return `${relationshipsService.formatRelationshipEndpoint(
			relationship.source_entity_kind,
			relationship.source_entity_id
		)} → ${relationshipsService.formatRelationshipEndpoint(
			relationship.target_entity_kind,
			relationship.target_entity_id
		)}`;
	}
</script>

<div
	class="widget-frame"
	class:editing={isLayoutEditing}
	data-widget-id="persons-relationship-review"
	data-widget-hidden={!isWidgetVisible('persons-relationship-review')}
>
	<WidgetEditChrome
		widgetId="persons-relationship-review"
		{isLayoutEditing}
		isSelected={false}
		onConfigure={() => {}}
	/>
	<section class="panel info-card relationship-review-panel" aria-busy={isLoading}>
		<header>
			<div>
				<span class="panel-kicker">{_('Relationships')}</span>
				<h2>{_('Relationship Review')}</h2>
			</div>
			<button type="button" title={_('Reload relationships')} onclick={() => void onReload()} disabled={isLoading}>
				<Icon icon="tabler:refresh" width="15" height="15" />
			</button>
		</header>

		{#if error}
			<div class="relationship-review-state error">
				<span>{error}</span>
				<button type="button" onclick={() => void onReload()} disabled={isLoading}>{_('Retry')}</button>
			</div>
		{:else if isLoading}
			<div class="relationship-review-state">
				<span>{_('Loading relationships')}</span>
			</div>
		{:else if suggestedRelationships.length === 0}
			<div class="relationship-review-state">
				<span>{_('No suggested relationships')}</span>
			</div>
		{:else}
			<div class="relationship-review-list">
				{#each suggestedRelationships as relationship}
					<article class="relationship-review-item">
						<div>
							<strong>{relationshipsService.formatRelationshipType(relationship.relationship_type)}</strong>
							<p>{relationshipPeer(relationship)}</p>
							<small>
								{_('Trust')}: {relationshipsService.formatRelationshipScore(relationship.trust_score)}
								· {_('Strength')}: {relationshipsService.formatRelationshipScore(relationship.strength_score)}
								· {_('Confidence')}: {relationshipsService.formatRelationshipScore(relationship.confidence)}
							</small>
						</div>
						<div class="relationship-review-actions">
							<button
								type="button"
								disabled={reviewingRelationshipId === relationship.relationship_id}
								onclick={() => void onReview(relationship, 'user_confirmed')}
							>
								<Icon icon="tabler:check" width="14" height="14" />
								{_('Confirm')}
							</button>
							<button
								type="button"
								disabled={reviewingRelationshipId === relationship.relationship_id}
								onclick={() => void onReview(relationship, 'user_rejected')}
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
</div>
