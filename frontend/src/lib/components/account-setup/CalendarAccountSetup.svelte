<script lang="ts">
	import {
		calendarProviderDefaultName,
		saveCalendarAccount as saveCalendarAccountService,
		type CalendarProvider,
		type CalendarWizardStep
	} from '$lib/services/accounts';
	import CalendarAccountWizard from './CalendarAccountWizard.svelte';

	interface Props {
		onAccountSaved?: () => Promise<void>;
	}

	let { onAccountSaved }: Props = $props();

	let calendarWizardStep = $state<CalendarWizardStep>('provider');
	let isSetupSubmitting = $state(false);
	let setupMessage = $state('');
	let setupError = $state('');
	let calendarAccountForm = $state({
		provider: 'local' as CalendarProvider,
		account_name: 'Local Calendar',
		email: ''
	});

	async function reloadAfterSave() {
		if (onAccountSaved) {
			await onAccountSaved();
		}
	}

	function continueCalendarWizard(provider?: CalendarProvider) {
		if (provider) {
			calendarAccountForm = {
				...calendarAccountForm,
				provider,
				account_name: calendarProviderDefaultName(provider)
			};
		}
		calendarWizardStep = 'details';
	}

	async function saveCalendarAccount() {
		isSetupSubmitting = true;
		setupMessage = '';
		setupError = '';
		try {
			const result = await saveCalendarAccountService(calendarAccountForm);
			setupMessage = result.message;
			setupError = result.error;
			if (!result.error) {
				await reloadAfterSave();
			}
		} finally {
			isSetupSubmitting = false;
		}
	}
</script>

<CalendarAccountWizard
	bind:calendarWizardStep
	bind:calendarAccountForm
	{isSetupSubmitting}
	{continueCalendarWizard}
	{saveCalendarAccount}
/>
{#if setupMessage}<p class="setup-state success">{setupMessage}</p>{/if}
{#if setupError}<p class="setup-state error">{setupError}</p>{/if}
