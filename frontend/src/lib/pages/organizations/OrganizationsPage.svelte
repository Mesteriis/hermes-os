<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import { loadOrganizations, loadPersons } from '$lib/services/persons';
	import type { EnrichedPerson, Organization } from '$lib/api';
	import OrganizationsList from './widgets/OrganizationsList.svelte';
	import OrganizationsDetail from './widgets/OrganizationsDetail.svelte';
	import './organizations.css';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let {
		isLayoutEditing,
		isWidgetVisible
	}: Props = $props();

	let organizations = $state<Organization[]>([]);
	let organizationsError = $state('');
	let isOrganizationsLoading = $state(false);
	let selectedOrganizationId = $state('');
	let persons = $state<EnrichedPerson[]>([]);

	let selectedOrganization = $derived(
		organizations.find((o) => o.organization_id === selectedOrganizationId) ?? organizations[0] ?? null
	);
	let orgPeople = $derived(
		persons.filter((p) =>
			p.linked_projects?.some((pid) =>
				selectedOrganization?.display_name && pid.includes(selectedOrganization.display_name)
			)
		).slice(0, 5)
	);

	$effect(() => {
		isOrganizationsLoading = true;
		loadOrganizations()
			.then((result) => {
				organizations = result.organizations;
				organizationsError = result.error;
				isOrganizationsLoading = false;
			})
			.catch((e: Error) => {
				organizationsError = e.message;
				isOrganizationsLoading = false;
			});

		loadPersons()
			.then((result) => {
				persons = result.persons;
			});
	});
</script>

<section class="organizations-page">
	<div class="view-header"><div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:building" width="28" height="28" /></span><div><h1>Companies</h1><p>All companies and organizations from your communications</p></div></div></div>
	{#if organizationsError}
		<p class="inline-error">{organizationsError}</p>
	{/if}
	<div class="org-layout">
		<OrganizationsList {organizations} {selectedOrganizationId} {isOrganizationsLoading} {isLayoutEditing} {isWidgetVisible} onSelectOrg={(id) => (selectedOrganizationId = id)} />
		<OrganizationsDetail selectedOrganization={selectedOrganization as Record<string, unknown> | null} orgPeople={orgPeople as unknown[]} {isLayoutEditing} {isWidgetVisible} />
	</div>
</section>
