<script lang="ts">
	import Icon from '@iconify/svelte';
	import CalendarAccountSetup from '$lib/components/account-setup/CalendarAccountSetup.svelte';
	import MailAccountSetup from '$lib/components/account-setup/MailAccountSetup.svelte';
	import TelegramAccountSetup from '$lib/components/account-setup/TelegramAccountSetup.svelte';
	import WhatsappAccountSetup from '$lib/components/account-setup/WhatsappAccountSetup.svelte';
	import { currentLocale, t } from '$lib/i18n';
	import { accountWizardTarget, type AccountWizardKind } from '$lib/stores/accountWizard';
	import type { TelegramCapabilitiesResponse } from '$lib/api';
	import type { MailService, Provider } from '$lib/services/accounts';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		isOpen: boolean;
		telegramCapabilities: TelegramCapabilitiesResponse | null;
		onAccountSaved?: () => Promise<void>;
	}

	let { isOpen = $bindable(), telegramCapabilities, onAccountSaved }: Props = $props();

	let accountWizardKind = $state<AccountWizardKind>('mail');
	let initialMailService = $state<MailService>('icloud');

	const modalKicker = $derived(
		accountWizardKind === 'mail'
			? _('Mail Account')
			: accountWizardKind === 'calendar'
				? _('Calendar Account')
				: accountWizardKind === 'telegram'
					? _('Telegram Account')
					: _('WhatsApp Account')
	);
	const modalTitle = $derived(
		accountWizardKind === 'mail'
			? _('New Mail Account')
			: accountWizardKind === 'calendar'
				? _('New Calendar Account')
				: accountWizardKind === 'telegram'
					? _('New Telegram Account')
					: _('New WhatsApp Account')
	);

	$effect(() => {
		if (!isOpen) {
			return;
		}
		const target = $accountWizardTarget;
		accountWizardKind = isMailTarget(target) ? 'mail' : target;
		if (isMailTarget(target)) {
			initialMailService = target;
		}
	});

	function isMailTarget(target: AccountWizardKind | Provider): target is Provider {
		return target === 'gmail' || target === 'icloud' || target === 'imap';
	}

	function closeAccountDrawer() {
		isOpen = false;
	}
</script>

{#if isOpen}
	<button
		type="button"
		class="drawer-backdrop modal-backdrop"
		aria-label={_('Close account setup')}
		onclick={closeAccountDrawer}
	></button>
	<div class="account-modal" role="dialog" aria-modal="true" aria-labelledby="account-setup-heading">
		<header>
			<div>
				<p>{modalKicker}</p>
				<h2 id="account-setup-heading">{modalTitle}</h2>
			</div>
			<button type="button" class="icon-button" onclick={closeAccountDrawer} aria-label={_('Close')}>
				<Icon icon="tabler:x" width="18" height="18" />
			</button>
		</header>

		{#if accountWizardKind === 'mail'}
			<MailAccountSetup {initialMailService} {onAccountSaved} />
		{:else if accountWizardKind === 'calendar'}
			<CalendarAccountSetup {onAccountSaved} />
		{:else if accountWizardKind === 'telegram'}
			<TelegramAccountSetup
				{isOpen}
				{telegramCapabilities}
				{closeAccountDrawer}
				{onAccountSaved}
			/>
		{:else if accountWizardKind === 'whatsapp'}
			<WhatsappAccountSetup {onAccountSaved} />
		{/if}
	</div>
{/if}
