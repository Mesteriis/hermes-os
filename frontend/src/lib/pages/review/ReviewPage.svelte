<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import * as reviewService from '$lib/services/review';
	import * as relationshipsService from '$lib/services/relationships';
	import * as decisionsService from '$lib/services/decisions';
	import * as obligationsService from '$lib/services/obligations';
	import * as contradictionsService from '$lib/services/contradictions';
	import type { Relationship } from '$lib/api';
	import type { ReviewWorkspace, ReviewWorkspaceItemAction } from '$lib/services/review';
	import './review.css';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();

	let workspace = $state<ReviewWorkspace | null>(null);
	let isLoading = $state(false);
	let actionError = $state('');
	let reviewingItemId = $state<string | null>(null);

	let relationships = $derived(workspace?.relationships.relationships ?? []);
	let decisions = $derived(workspace?.decisions.decisions ?? []);
	let obligations = $derived(workspace?.obligations.obligations ?? []);
	let contradictions = $derived(workspace?.contradictions.observations ?? []);
	let totalSuggestedCount = $derived(workspace?.totalSuggestedCount ?? 0);
	let reviewError = $derived([workspace?.error, actionError].filter(Boolean).join(' · '));

	async function loadWorkspace() {
		isLoading = true;
		workspace = await reviewService.loadReviewWorkspace();
		actionError = '';
		isLoading = false;
	}

	async function reviewWorkspaceAction(action: ReviewWorkspaceItemAction) {
		reviewingItemId = reviewService.reviewWorkspaceItemKey(action);
		const result = await reviewService.reviewWorkspaceItem(action);
		if (result.error) {
			actionError = result.error;
		} else {
			await loadWorkspace();
		}
		reviewingItemId = null;
	}

	function relationshipPeer(relationship: Relationship) {
		return `${relationshipsService.formatRelationshipEndpoint(
			relationship.source_entity_kind,
			relationship.source_entity_id
		)} -> ${relationshipsService.formatRelationshipEndpoint(
			relationship.target_entity_kind,
			relationship.target_entity_id
		)}`;
	}

	$effect(() => {
		loadWorkspace();
	});
</script>

<section class="review-page">
	<div class="view-header">
		<div class="view-title-with-icon">
			<span class="hero-mark small">
				<Icon icon="tabler:clipboard-check" width="28" height="28" />
			</span>
			<div>
				<h1>{_('Review')}</h1>
				<p>{_('Suggested context updates')}</p>
			</div>
		</div>
		<button type="button" class="primary-button" onclick={() => void loadWorkspace()} disabled={isLoading}>
			<Icon icon="tabler:refresh" width="16" height="16" />
			{_('Refresh')}
		</button>
	</div>

	<section
		class="widget-frame review-overview"
		class:editing={isLayoutEditing}
		data-widget-id="review-overview"
		data-widget-hidden={!isWidgetVisible('review-overview')}
	>
		<WidgetEditChrome
			widgetId="review-overview"
			{isLayoutEditing}
			isSelected={false}
			onConfigure={() => {}}
		/>
		<div class="review-metrics">
			<article><span>{_('Suggested')}</span><strong>{totalSuggestedCount}</strong><small>{_('Open items')}</small></article>
			<article><span>{_('Relationships')}</span><strong>{relationships.length}</strong><small>{_('Persona graph')}</small></article>
			<article><span>{_('Decisions')}</span><strong>{decisions.length}</strong><small>{_('Source-backed')}</small></article>
			<article><span>{_('Obligations')}</span><strong>{obligations.length}</strong><small>{_('Commitments')}</small></article>
			<article><span>{_('Polygraph')}</span><strong>{contradictions.length}</strong><small>{_('Contradictions')}</small></article>
		</div>
	</section>

	{#if reviewError}
		<p class="inline-error">{reviewError}</p>
	{/if}

	<div class="review-board">
		<section
			class="widget-frame review-queue-panel"
			class:editing={isLayoutEditing}
			data-widget-id="review-relationships"
			data-widget-hidden={!isWidgetVisible('review-relationships')}
			aria-busy={isLoading}
		>
			<WidgetEditChrome
				widgetId="review-relationships"
				{isLayoutEditing}
				isSelected={false}
				onConfigure={() => {}}
			/>
			<header>
				<div>
					<span class="panel-kicker">{_('Relationships')}</span>
					<h2>{_('Relationship Review')}</h2>
				</div>
			</header>
			{#if isLoading && relationships.length === 0}
				<div class="review-empty">{_('Loading review items')}</div>
			{:else if relationships.length === 0}
				<div class="review-empty">{_('No suggested relationships')}</div>
			{:else}
				<div class="review-list">
					{#each relationships as relationship}
						<article class="review-item">
							<div>
								<strong>{relationshipsService.formatRelationshipType(relationship.relationship_type)}</strong>
								<p>{relationshipPeer(relationship)}</p>
								<small>
									{_('Trust')}: {relationshipsService.formatRelationshipScore(relationship.trust_score)}
									· {_('Strength')}: {relationshipsService.formatRelationshipScore(relationship.strength_score)}
								</small>
							</div>
							<div class="review-actions">
								<button
									type="button"
									disabled={reviewingItemId === `relationship:${relationship.relationship_id}`}
									onclick={() =>
										void reviewWorkspaceAction({
											kind: 'relationship',
											item: relationship,
											reviewState: 'user_confirmed'
										})}
								>
									<Icon icon="tabler:check" width="14" height="14" />{_('Confirm')}
								</button>
								<button
									type="button"
									disabled={reviewingItemId === `relationship:${relationship.relationship_id}`}
									onclick={() =>
										void reviewWorkspaceAction({
											kind: 'relationship',
											item: relationship,
											reviewState: 'user_rejected'
										})}
								>
									<Icon icon="tabler:x" width="14" height="14" />{_('Reject')}
								</button>
							</div>
						</article>
					{/each}
				</div>
			{/if}
		</section>

		<section
			class="widget-frame review-queue-panel"
			class:editing={isLayoutEditing}
			data-widget-id="review-decisions"
			data-widget-hidden={!isWidgetVisible('review-decisions')}
			aria-busy={isLoading}
		>
			<WidgetEditChrome
				widgetId="review-decisions"
				{isLayoutEditing}
				isSelected={false}
				onConfigure={() => {}}
			/>
			<header>
				<div>
					<span class="panel-kicker">{_('Decisions')}</span>
					<h2>{_('Decision Review')}</h2>
				</div>
			</header>
			{#if isLoading && decisions.length === 0}
				<div class="review-empty">{_('Loading review items')}</div>
			{:else if decisions.length === 0}
				<div class="review-empty">{_('No open decisions')}</div>
			{:else}
				<div class="review-list">
					{#each decisions as decision}
						<article class="review-item">
							<div>
								<strong>{decision.title}</strong>
								<p>{decision.rationale}</p>
								<small>{decisionsService.formatDecisionTime(decision.decided_at)}</small>
							</div>
							<div class="review-actions">
								<button
									type="button"
									disabled={reviewingItemId === `decision:${decision.decision_id}`}
									onclick={() =>
										void reviewWorkspaceAction({
											kind: 'decision',
											item: decision,
											reviewState: 'user_confirmed'
										})}
								>
									<Icon icon="tabler:check" width="14" height="14" />{_('Confirm')}
								</button>
								<button
									type="button"
									disabled={reviewingItemId === `decision:${decision.decision_id}`}
									onclick={() =>
										void reviewWorkspaceAction({
											kind: 'decision',
											item: decision,
											reviewState: 'user_rejected'
										})}
								>
									<Icon icon="tabler:x" width="14" height="14" />{_('Reject')}
								</button>
							</div>
						</article>
					{/each}
				</div>
			{/if}
		</section>

		<section
			class="widget-frame review-queue-panel"
			class:editing={isLayoutEditing}
			data-widget-id="review-obligations"
			data-widget-hidden={!isWidgetVisible('review-obligations')}
			aria-busy={isLoading}
		>
			<WidgetEditChrome
				widgetId="review-obligations"
				{isLayoutEditing}
				isSelected={false}
				onConfigure={() => {}}
			/>
			<header>
				<div>
					<span class="panel-kicker">{_('Obligations')}</span>
					<h2>{_('Obligation Review')}</h2>
				</div>
			</header>
			{#if isLoading && obligations.length === 0}
				<div class="review-empty">{_('Loading review items')}</div>
			{:else if obligations.length === 0}
				<div class="review-empty">{_('No open obligations')}</div>
			{:else}
				<div class="review-list">
					{#each obligations as obligation}
						<article class="review-item">
							<div>
								<strong>{obligation.statement}</strong>
								<p>{obligationsService.formatObligationEntity(obligation.obligated_entity_kind, obligation.obligated_entity_id)}</p>
								<small>{obligationsService.formatObligationDueTime(obligation.due_at)}</small>
							</div>
							<div class="review-actions">
								<button
									type="button"
									disabled={reviewingItemId === `obligation:${obligation.obligation_id}`}
									onclick={() =>
										void reviewWorkspaceAction({
											kind: 'obligation',
											item: obligation,
											reviewState: 'user_confirmed'
										})}
								>
									<Icon icon="tabler:check" width="14" height="14" />{_('Confirm')}
								</button>
								<button
									type="button"
									disabled={reviewingItemId === `obligation:${obligation.obligation_id}`}
									onclick={() =>
										void reviewWorkspaceAction({
											kind: 'obligation',
											item: obligation,
											reviewState: 'user_rejected'
										})}
								>
									<Icon icon="tabler:x" width="14" height="14" />{_('Reject')}
								</button>
							</div>
						</article>
					{/each}
				</div>
			{/if}
		</section>

		<section
			class="widget-frame review-queue-panel"
			class:editing={isLayoutEditing}
			data-widget-id="review-polygraph"
			data-widget-hidden={!isWidgetVisible('review-polygraph')}
			aria-busy={isLoading}
		>
			<WidgetEditChrome
				widgetId="review-polygraph"
				{isLayoutEditing}
				isSelected={false}
				onConfigure={() => {}}
			/>
			<header>
				<div>
					<span class="panel-kicker">{_('Polygraph')}</span>
					<h2>{_('Contradiction Review')}</h2>
				</div>
			</header>
			{#if isLoading && contradictions.length === 0}
				<div class="review-empty">{_('Loading review items')}</div>
			{:else if contradictions.length === 0}
				<div class="review-empty">{_('No open contradictions')}</div>
			{:else}
				<div class="review-list">
					{#each contradictions as observation}
						<article class="review-item">
							<div>
								<strong>{contradictionsService.formatContradictionClaim(observation)}</strong>
								<p>{contradictionsService.formatContradictionSource(observation.new_source_kind, observation.new_source_id)}</p>
								<small>{contradictionsService.formatContradictionTime(observation.updated_at)}</small>
							</div>
							<div class="review-actions">
								<button
									type="button"
									disabled={reviewingItemId === `contradiction:${observation.observation_id}`}
									onclick={() =>
										void reviewWorkspaceAction({
											kind: 'contradiction',
											item: observation,
											reviewState: 'user_confirmed'
										})}
								>
									<Icon icon="tabler:check" width="14" height="14" />{_('Confirm')}
								</button>
								<button
									type="button"
									disabled={reviewingItemId === `contradiction:${observation.observation_id}`}
									onclick={() =>
										void reviewWorkspaceAction({
											kind: 'contradiction',
											item: observation,
											reviewState: 'user_rejected'
										})}
								>
									<Icon icon="tabler:x" width="14" height="14" />{_('Reject')}
								</button>
							</div>
						</article>
					{/each}
				</div>
			{/if}
		</section>
	</div>
</section>
