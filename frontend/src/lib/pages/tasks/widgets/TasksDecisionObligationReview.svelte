<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import * as decisionsService from '$lib/services/decisions';
	import * as obligationsService from '$lib/services/obligations';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type {
		Decision,
		DecisionEntityKind,
		DecisionReviewState,
		Obligation,
		ObligationReviewState
	} from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	const entityKindOptions: DecisionEntityKind[] = [
		'project',
		'task',
		'persona',
		'communication',
		'document',
		'event',
		'organization',
		'knowledge'
	];

	interface Props {
		decisions: Decision[];
		obligations: Obligation[];
		entityKind: DecisionEntityKind;
		entityId: string;
		isLoading: boolean;
		error: string;
		reviewingItemId: string | null;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onEntityKindChange: (entityKind: DecisionEntityKind) => void;
		onEntityIdChange: (entityId: string) => void;
		onReload: () => Promise<void>;
		onReviewDecision: (
			decision: Decision,
			reviewState: Exclude<DecisionReviewState, 'suggested'>
		) => Promise<void>;
		onReviewObligation: (
			obligation: Obligation,
			reviewState: Exclude<ObligationReviewState, 'suggested'>
		) => Promise<void>;
	}

	let {
		decisions,
		obligations,
		entityKind,
		entityId,
		isLoading,
		error,
		reviewingItemId,
		isLayoutEditing,
		isWidgetVisible,
		onEntityKindChange,
		onEntityIdChange,
		onReload,
		onReviewDecision,
		onReviewObligation
	}: Props = $props();

	let hasScope = $derived(entityId.trim().length > 0);
	let reviewItemCount = $derived(decisions.length + obligations.length);

	function decisionReviewId(decision: Decision) {
		return `decision:${decision.decision_id}`;
	}

	function obligationReviewId(obligation: Obligation) {
		return `obligation:${obligation.obligation_id}`;
	}
</script>

<section
	class="widget-frame task-context-review-panel"
	class:editing={isLayoutEditing}
	data-widget-id="tasks-decision-obligation-review"
	data-widget-hidden={!isWidgetVisible('tasks-decision-obligation-review')}
	aria-busy={isLoading}
>
	<WidgetEditChrome
		widgetId="tasks-decision-obligation-review"
		{isLayoutEditing}
		isSelected={false}
		onConfigure={() => {}}
	/>
	<header>
		<div>
			<span class="panel-kicker">{_('Context')}</span>
			<h2>{_('Decision & Obligation Review')}</h2>
		</div>
		<button type="button" title={_('Reload review items')} onclick={() => void onReload()} disabled={isLoading}>
			<Icon icon="tabler:refresh" width="15" height="15" />
		</button>
	</header>

	<div class="task-context-review-scope">
		<select
			aria-label={_('Entity kind')}
			value={entityKind}
			onchange={(event) => onEntityKindChange((event.currentTarget as HTMLSelectElement).value as DecisionEntityKind)}
		>
			{#each entityKindOptions as option}
				<option value={option}>{_(option)}</option>
			{/each}
		</select>
		<input
			aria-label={_('Entity id')}
			value={entityId}
			placeholder={_('Entity id')}
			oninput={(event) => onEntityIdChange((event.currentTarget as HTMLInputElement).value)}
		/>
	</div>

	{#if error}
		<div class="task-context-review-state error">
			<span>{error}</span>
			<button type="button" onclick={() => void onReload()} disabled={isLoading}>{_('Retry')}</button>
		</div>
	{:else if isLoading}
		<div class="task-context-review-state">
			<span>{hasScope ? _('Loading review items') : _('Loading global review items')}</span>
		</div>
	{:else if reviewItemCount === 0}
		<div class="task-context-review-state">
			<span>{_('No open decisions or obligations')}</span>
		</div>
	{:else}
		<div class="task-context-review-list">
			{#each decisions as decision}
				<article class="task-context-review-item">
					<div>
						<span class="panel-kicker">{_('Decision')}</span>
						<strong>{decision.title}</strong>
						<p>{decision.rationale}</p>
						<small>{decisionsService.formatDecisionEntity(decision.decided_by_entity_kind, decision.decided_by_entity_id)} · {decisionsService.formatDecisionTime(decision.decided_at)}</small>
					</div>
					<div class="task-context-review-actions">
						<button
							type="button"
							disabled={reviewingItemId === decisionReviewId(decision)}
							onclick={() => void onReviewDecision(decision, 'user_confirmed')}
						>
							<Icon icon="tabler:check" width="14" height="14" />
							{_('Confirm')}
						</button>
						<button
							type="button"
							disabled={reviewingItemId === decisionReviewId(decision)}
							onclick={() => void onReviewDecision(decision, 'user_rejected')}
						>
							<Icon icon="tabler:x" width="14" height="14" />
							{_('Reject')}
						</button>
					</div>
				</article>
			{/each}

			{#each obligations as obligation}
				<article class="task-context-review-item">
					<div>
						<span class="panel-kicker">{_('Obligation')}</span>
						<strong>{obligation.statement}</strong>
						<p>{obligationsService.formatObligationEntity(obligation.obligated_entity_kind, obligation.obligated_entity_id)}</p>
						<small>{_(obligation.risk_state)} · {obligationsService.formatObligationDueTime(obligation.due_at)}</small>
					</div>
					<div class="task-context-review-actions">
						<button
							type="button"
							disabled={reviewingItemId === obligationReviewId(obligation)}
							onclick={() => void onReviewObligation(obligation, 'user_confirmed')}
						>
							<Icon icon="tabler:check" width="14" height="14" />
							{_('Confirm')}
						</button>
						<button
							type="button"
							disabled={reviewingItemId === obligationReviewId(obligation)}
							onclick={() => void onReviewObligation(obligation, 'user_rejected')}
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
