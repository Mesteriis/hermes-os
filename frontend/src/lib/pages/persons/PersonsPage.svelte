<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import * as personsService from '$lib/services/persons';
	import type { PersonIdentityCandidate } from '$lib/api';
	import PersonsList from './widgets/PersonsList.svelte';
	import PersonsDetail from './widgets/PersonsDetail.svelte';
	import PersonsIdentityReview from './widgets/PersonsIdentityReview.svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	type FeedItem = {
		icon: string;
		title: string;
		meta: string;
		time: string;
		tag?: string;
		tone?: string;
	};

	type ProjectItem = {
		name: string;
		kind: string;
		progress: number;
		icon: string;
		tone: string;
	};

	type DocItem = {
		icon: string;
		name: string;
		size: string;
		date: string;
	};

	type PersonItem = {
		name: string;
		role: string;
		company: string;
		channel?: string;
		status?: string;
	};

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();

	let persons = $state<PersonItem[]>([]);
	let personList = $state<PersonItem[]>([]);
	let selectedPersonIndex = $state(0);
	let identityCandidates = $state<PersonIdentityCandidate[]>([]);
	let isIdentityCandidatesLoading = $state(false);
	let identityCandidatesError = $state('');

	let selectedPerson = $derived(personList[selectedPersonIndex] ?? personList[0]);

	let suggestedIdentityCandidates = $derived(
		identityCandidates.filter((item) => item.review_state === 'suggested')
	);
	let confirmedMergeIdentityCandidates = $derived(
		identityCandidates.filter(
			(item) =>
				item.candidate_kind === 'merge_persons' &&
				item.review_state === 'user_confirmed' &&
				!confirmedSplitCandidateForMerge(item)
		)
	);

	function identityConfidence(item: PersonIdentityCandidate) {
		return `${Math.round(item.confidence * 100)}%`;
	}

	function confirmedSplitCandidateForMerge(candidate: PersonIdentityCandidate) {
		return splitCandidateForMerge(candidate, 'user_confirmed');
	}

	function splitCandidateForConfirmedMerge(candidate: PersonIdentityCandidate) {
		return splitCandidateForMerge(candidate, 'suggested');
	}

	function splitCandidateForMerge(
		candidate: PersonIdentityCandidate,
		reviewState: string
	): PersonIdentityCandidate | null {
		if (!candidate.right_person_id) return null;
		const pairKey = personIdentityPairKey(candidate.left_person_id, candidate.right_person_id);
		return (
			identityCandidates.find(
				(item) =>
					item.candidate_kind === 'split_person' &&
					item.review_state === reviewState &&
					item.right_person_id !== null &&
					personIdentityPairKey(item.left_person_id, item.right_person_id) === pairKey
			) ?? null
		);
	}

	function personIdentityPairKey(leftPersonId: string, rightPersonId: string) {
		return leftPersonId <= rightPersonId
			? `${leftPersonId}:${rightPersonId}`
			: `${rightPersonId}:${leftPersonId}`;
	}

	async function setIdentityCandidateReview(
		candidate: PersonIdentityCandidate,
		state: string
	) {
		const result = await personsService.setIdentityCandidateReview(
			candidate,
			state as 'suggested' | 'user_confirmed' | 'user_rejected'
		);
		if (result.error) {
			identityCandidatesError = result.error;
		} else {
			await loadIdentityCandidates();
		}
	}

	async function splitConfirmedIdentityMerge(candidate: PersonIdentityCandidate) {
		const splitCandidate = splitCandidateForConfirmedMerge(candidate);
		const result = await personsService.splitConfirmedIdentityMerge(candidate, splitCandidate);
		if (result.error) {
			identityCandidatesError = result.error;
		} else {
			await loadIdentityCandidates();
		}
	}

	async function loadPersons() {
		const result = await personsService.loadPersons();
		persons = result.persons.map((p) => ({
			name: p.display_name,
			role: p.preferred_channel || _('Contact'),
			company: p.email_address,
			status: p.last_interaction_at ? _('Online') : undefined,
			channel: p.preferred_channel ?? undefined
		}));
		personList = [...persons];
	}

	async function loadIdentityCandidates() {
		isIdentityCandidatesLoading = true;
		const result = await personsService.loadIdentityCandidates();
		identityCandidates = result.candidates;
		identityCandidatesError = result.error;
		isIdentityCandidatesLoading = false;
	}

	$effect(() => {
		loadPersons();
		loadIdentityCandidates();
	});
</script>

<section class="persons-page">
	<div class="persons-layout">
		<PersonsList
			{personList}
			{selectedPersonIndex}
			{isLayoutEditing}
			{isWidgetVisible}
			onSelectPerson={(index) => { selectedPersonIndex = index; }}
		/>
		<PersonsDetail
			{selectedPerson}
			whatsNew={[]}
			projects={[]}
			{isLayoutEditing}
			{isWidgetVisible}
		/>
		<aside class="stacked-rail">
			<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-ai-summary" data-widget-hidden={!isWidgetVisible('persons-ai-summary')}>
				<WidgetEditChrome widgetId="persons-ai-summary" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
				<section class="panel info-card"><h2>{_('AI Summary')}</h2><p>{_('John is a key strategic partner and decision maker. You have a strong professional relationship with frequent communication across multiple projects.')}</p></section>
			</div>
			<PersonsIdentityReview
				{suggestedIdentityCandidates}
				{confirmedMergeIdentityCandidates}
				{isIdentityCandidatesLoading}
				{identityCandidatesError}
				{isLayoutEditing}
				{isWidgetVisible}
				{identityConfidence}
				{setIdentityCandidateReview}
				{splitConfirmedIdentityMerge}
				{splitCandidateForConfirmedMerge}
			/>
			<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-related-documents" data-widget-hidden={!isWidgetVisible('persons-related-documents')}>
				<WidgetEditChrome widgetId="persons-related-documents" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
				<section class="panel info-card"><h2>{_('Related Documents')}</h2><p>{_('Documents will appear here when processing is complete.')}</p></section>
			</div>
			<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-recent-notes" data-widget-hidden={!isWidgetVisible('persons-recent-notes')}>
				<WidgetEditChrome widgetId="persons-recent-notes" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
				<section class="panel info-card"><h2>{_('Recent Notes')}</h2><p>{_('Discussed expansion to EU market')}</p><p>{_('Prefers email for official communication')}</p><p>{_('Interested in AI/ML integration')}</p></section>
			</div>
		</aside>
	</div>

</section>
