<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { PersonIdentityCandidate } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		suggestedIdentityCandidates: PersonIdentityCandidate[];
		confirmedMergeIdentityCandidates: PersonIdentityCandidate[];
		isIdentityCandidatesLoading: boolean;
		identityCandidatesError: string;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		identityConfidence: (candidate: PersonIdentityCandidate) => string;
		setIdentityCandidateReview: (candidate: PersonIdentityCandidate, state: string) => Promise<void>;
		splitConfirmedIdentityMerge: (candidate: PersonIdentityCandidate) => Promise<void>;
		splitCandidateForConfirmedMerge: (candidate: PersonIdentityCandidate) => PersonIdentityCandidate | null;
	}

	let {
		suggestedIdentityCandidates,
		confirmedMergeIdentityCandidates,
		isIdentityCandidatesLoading,
		identityCandidatesError,
		isLayoutEditing,
		isWidgetVisible,
		identityConfidence,
		setIdentityCandidateReview,
		splitConfirmedIdentityMerge,
		splitCandidateForConfirmedMerge
	}: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-identity-review" data-widget-hidden={!isWidgetVisible('persons-identity-review')}>
	<WidgetEditChrome widgetId="persons-identity-review" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card">
		<h2>{_('Person Identity Review')}</h2>
		<p class="identity-note">{_('Person merges are only suggested and are not applied until confirmed.')}</p>
		{#if isIdentityCandidatesLoading}
			<p class="inline-copy">{_('Loading identity suggestions…')}</p>
		{:else if identityCandidatesError}
			<p class="inline-error">{identityCandidatesError}</p>
		{:else if suggestedIdentityCandidates.length === 0 && confirmedMergeIdentityCandidates.length === 0}
			<p class="inline-copy">{_('No identity suggestions right now.')}</p>
		{:else}
			{#each suggestedIdentityCandidates as candidate}
				<div class="identity-candidate-row">
					<div>
						<strong>{candidate.candidate_kind}</strong>
						<p>{candidate.evidence_summary}</p>
						<small>Left: {candidate.left_person_id}</small>
						<small>Right: {candidate.right_person_id ?? _('N/A')}</small>
						<small>{_('Confidence')}: {identityConfidence(candidate)} · {candidate.review_state}</small>
					</div>
					<div class="identity-actions">
						<button type="button" onclick={() => void setIdentityCandidateReview(candidate, 'user_confirmed')}>
				<Icon icon="tabler:check" width="15" height="15" />
					{_('Confirm')}
						</button>
						<button type="button" onclick={() => void setIdentityCandidateReview(candidate, 'user_rejected')}>
				<Icon icon="tabler:x" width="15" height="15" />
					{_('Reject')}
						</button>
					</div>
				</div>
			{/each}
			{#each confirmedMergeIdentityCandidates as candidate}
				{@const splitCandidate = splitCandidateForConfirmedMerge(candidate)}
				<div class="identity-candidate-row">
					<div>
					<strong>{candidate.candidate_kind}</strong>
					<p>{candidate.evidence_summary}</p>
					<small>Left: {candidate.left_person_id}</small>
					<small>Right: {candidate.right_person_id ?? _('N/A')}</small>
					<small>{_('Confidence')}: {identityConfidence(candidate)} · {candidate.review_state}</small>
					</div>
					<div class="identity-actions">
						<button
							type="button"
							disabled={splitCandidate === null}
							title={splitCandidate === null
								? _('Refresh identity candidates to create a split review for this confirmed link')
								: undefined}
							onclick={() => void splitConfirmedIdentityMerge(candidate)}
						>
				<Icon icon="tabler:arrows-split" width="15" height="15" />
					{_('Split')}
						</button>
					</div>
				</div>
			{/each}
		{/if}
	</section>
</div>
