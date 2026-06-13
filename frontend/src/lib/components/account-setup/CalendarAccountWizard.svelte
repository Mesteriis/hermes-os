<script lang="ts">
	import Icon from '@iconify/svelte';
	import HermesSelect from '$lib/components/shared/HermesSelect.svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { CalendarProvider, CalendarWizardStep } from '$lib/services/accounts';

	const _ = (key: string) => t($currentLocale, key);

	type CalendarAccountForm = {
		provider: CalendarProvider;
		account_name: string;
		email: string;
	};

	interface Props {
		calendarWizardStep: CalendarWizardStep;
		calendarAccountForm: CalendarAccountForm;
		isSetupSubmitting: boolean;
		continueCalendarWizard: (provider?: CalendarProvider) => void;
		saveCalendarAccount: () => Promise<void>;
	}

	let {
		calendarWizardStep = $bindable(),
		calendarAccountForm = $bindable(),
		isSetupSubmitting,
		continueCalendarWizard,
		saveCalendarAccount
	}: Props = $props();

	const calendarProviderOptions = $derived(
		[
			{ value: 'local', label: 'Local' },
			{ value: 'google', label: 'Google Calendar' },
			{ value: 'microsoft', label: 'Microsoft 365' },
			{ value: 'apple', label: 'Apple Calendar' },
			{ value: 'caldav', label: 'CalDAV' },
			{ value: 'ics', label: 'ICS Feed' }
		].map((provider) => ({ value: provider.value, label: _(provider.label) }))
	);
</script>

<div class="wizard-progress" aria-label={_('Calendar setup steps')}>
	<span class:active={calendarWizardStep === 'provider'}>{_('1. Provider')}</span>
	<span class:active={calendarWizardStep === 'details'}>{_('2. Details')}</span>
</div>

{#if calendarWizardStep === 'provider'}
	<div class="wizard-step">
		<div class="wizard-choice-grid">
			<button type="button" onclick={() => continueCalendarWizard('local')}><Icon icon="tabler:calendar" width="28" height="28" /><strong>{_('Local')}</strong></button>
			<button type="button" onclick={() => continueCalendarWizard('google')}><Icon icon="tabler:brand-google" width="28" height="28" /><strong>{_('Google Calendar')}</strong></button>
			<button type="button" onclick={() => continueCalendarWizard('microsoft')}><Icon icon="tabler:brand-office" width="28" height="28" /><strong>{_('Microsoft 365')}</strong></button>
			<button type="button" onclick={() => continueCalendarWizard('apple')}><Icon icon="tabler:apple" width="28" height="28" /><strong>{_('Apple Calendar')}</strong></button>
			<button type="button" onclick={() => continueCalendarWizard('caldav')}><Icon icon="tabler:server" width="28" height="28" /><strong>{_('CalDAV')}</strong></button>
			<button type="button" onclick={() => continueCalendarWizard('ics')}><Icon icon="tabler:rss" width="28" height="28" /><strong>{_('ICS Feed')}</strong></button>
		</div>
	</div>
{:else}
	<form class="setup-form" onsubmit={(event) => { event.preventDefault(); void saveCalendarAccount(); }}>
		<button type="button" class="wizard-back wide" onclick={() => (calendarWizardStep = 'provider')}><Icon icon="tabler:arrow-left" width="15" height="15" />{_('Provider')}</button>
		<label>
			<span>{_('Provider')}</span>
			<HermesSelect
				value={calendarAccountForm.provider}
				options={calendarProviderOptions}
				placeholder={_('Provider')}
				searchPlaceholder={_('Search providers...')}
				emptyLabel={_('No options')}
				ariaLabel={_('Provider')}
				searchable={false}
				onChange={(nextValue) => (calendarAccountForm.provider = nextValue as CalendarProvider)}
			/>
		</label>
		<label><span>{_('Account name')}</span><input bind:value={calendarAccountForm.account_name} autocomplete="off" /></label>
		<label class="wide"><span>{_('Email or owner')}</span><input bind:value={calendarAccountForm.email} autocomplete="email" /></label>
		<div class="form-actions wide"><button type="submit" disabled={isSetupSubmitting || !calendarAccountForm.account_name.trim()}>{_('Save Calendar')}</button></div>
	</form>
{/if}
