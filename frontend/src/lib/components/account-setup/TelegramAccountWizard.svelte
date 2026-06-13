<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { TelegramProviderKind, TelegramQrLoginStatusResponse } from '$lib/api';
	import type {
		TelegramAccountDraft,
		TelegramAuthMethod,
		TelegramWizardStep
	} from '$lib/services/accounts';
	import TelegramQrLoginPanel from './TelegramQrLoginPanel.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		telegramWizardStep: TelegramWizardStep;
		telegramAuthMethod: TelegramAuthMethod;
		telegramAccountForm: TelegramAccountDraft;
		telegramQrLogin: TelegramQrLoginStatusResponse | null;
		telegramQrPassword: string;
		telegramQrIsPreparing: boolean;
		telegramNeedsFormAppCredentials: boolean;
		isTelegramActionSubmitting: boolean;
		telegramWizardNote: string;
		selectTelegramProviderKind: (providerKind: TelegramProviderKind) => void;
		selectTelegramAuthMethod: (method: TelegramAuthMethod) => void;
		continueTelegramWizard: (nextStep: TelegramWizardStep) => void;
		submitTelegramQrStepFromWizard: () => void;
		saveTelegramAccountFromWizard: () => Promise<void>;
		closeAccountDrawer: () => void;
		telegramQrStatusLabel: (status: string) => string;
	}

	let {
		telegramWizardStep,
		telegramAuthMethod,
		telegramAccountForm = $bindable(),
		telegramQrLogin,
		telegramQrPassword = $bindable(),
		telegramQrIsPreparing,
		telegramNeedsFormAppCredentials,
		isTelegramActionSubmitting,
		telegramWizardNote,
		selectTelegramProviderKind,
		selectTelegramAuthMethod,
		continueTelegramWizard,
		submitTelegramQrStepFromWizard,
		saveTelegramAccountFromWizard,
		closeAccountDrawer,
		telegramQrStatusLabel
	}: Props = $props();
</script>

<div class="wizard-progress" aria-label={_('Telegram setup steps')}>
	<span class:active={telegramWizardStep === 'account'}>{_('1. Account')}</span>
	<span class:active={telegramWizardStep === 'auth'}>{_('2. Login')}</span>
	<span class:active={telegramWizardStep === 'details'}>{_('3. Details')}</span>
</div>

{#if telegramWizardStep === 'account'}
	<div class="wizard-step">
		<div class="wizard-choice-grid two">
			<button type="button" class:active={telegramAccountForm.provider_kind === 'telegram_user'} onclick={() => { selectTelegramProviderKind('telegram_user'); selectTelegramAuthMethod('phone'); continueTelegramWizard('auth'); }}><Icon icon="tabler:user" width="30" height="30" /><strong>{_('User Account')}</strong><span>{_('Phone or QR login')}</span></button>
			<button type="button" class:active={telegramAccountForm.provider_kind === 'telegram_bot'} onclick={() => { selectTelegramProviderKind('telegram_bot'); selectTelegramAuthMethod('bot_token'); continueTelegramWizard('auth'); }}><Icon icon="tabler:robot" width="30" height="30" /><strong>{_('Bot Account')}</strong><span>{_('Bot token')}</span></button>
		</div>
	</div>
{:else if telegramWizardStep === 'auth'}
	<div class="wizard-step">
		<button type="button" class="wizard-back" onclick={() => continueTelegramWizard('account')}><Icon icon="tabler:arrow-left" width="15" height="15" />{_('Account')}</button>
		<div class="wizard-choice-grid">
			{#if telegramAccountForm.provider_kind === 'telegram_user'}
				<button type="button" class:active={telegramAuthMethod === 'phone'} onclick={() => { selectTelegramAuthMethod('phone'); continueTelegramWizard('details'); }}><Icon icon="tabler:phone" width="28" height="28" /><strong>{_('Phone Number')}</strong></button>
				<button type="button" class:active={telegramAuthMethod === 'qr'} onclick={() => { selectTelegramAuthMethod('qr'); continueTelegramWizard('details'); }}><Icon icon="tabler:qrcode" width="28" height="28" /><strong>{_('QR Code')}</strong></button>
			{/if}
			{#if telegramAccountForm.provider_kind === 'telegram_bot'}
				<button type="button" class:active={telegramAuthMethod === 'bot_token'} onclick={() => { selectTelegramAuthMethod('bot_token'); continueTelegramWizard('details'); }}><Icon icon="tabler:key" width="28" height="28" /><strong>{_('Bot Token')}</strong></button>
			{/if}
			<button type="button" class:active={telegramAuthMethod === 'fixture'} onclick={() => { selectTelegramAuthMethod('fixture'); continueTelegramWizard('details'); }}><Icon icon="tabler:flask" width="28" height="28" /><strong>{_('Fixture')}</strong></button>
		</div>
	</div>
{:else}
	<form class="setup-form" class:telegram-qr-setup-form={telegramAuthMethod === 'qr'} onsubmit={(event) => { event.preventDefault(); telegramAuthMethod === 'qr' ? submitTelegramQrStepFromWizard() : void saveTelegramAccountFromWizard(); }}>
		<button type="button" class="wizard-back wide" onclick={() => continueTelegramWizard('auth')}><Icon icon="tabler:arrow-left" width="15" height="15" />{_('Login')}</button>
		{#if telegramAuthMethod !== 'qr'}
			<label><span>{_('Account ID')}</span><input bind:value={telegramAccountForm.account_id} autocomplete="off" /></label>
			<label><span>{_('Display name')}</span><input bind:value={telegramAccountForm.display_name} autocomplete="off" /></label>
		{/if}
		{#if telegramAuthMethod === 'phone'}
			<label class="wide"><span>{_('Phone number')}</span><input bind:value={telegramAccountForm.external_account_id} autocomplete="tel" placeholder={_('+15551234567')} /></label>
		{:else if telegramAuthMethod === 'qr'}
			<TelegramQrLoginPanel
				{telegramQrLogin}
				{telegramQrIsPreparing}
				accountId={telegramAccountForm.account_id}
				{telegramQrStatusLabel}
			/>
			{#if telegramQrLogin?.status === 'waiting_password'}
				<label class="wide"><span>{_('Telegram Cloud Password')}</span><input bind:value={telegramQrPassword} type="password" autocomplete="current-password" /></label>
			{/if}
		{:else}
			<label class="wide"><span>{_('External ID')}</span><input bind:value={telegramAccountForm.external_account_id} autocomplete="off" /></label>
		{/if}
		{#if telegramNeedsFormAppCredentials}
			<label><span>{_('API ID')}</span><input bind:value={telegramAccountForm.api_id} inputmode="numeric" autocomplete="off" /></label>
			<label><span>{_('API hash')}</span><input bind:value={telegramAccountForm.api_hash} type="password" autocomplete="off" /></label>
			<label class="wide"><span>{_('Session encryption key')}</span><input bind:value={telegramAccountForm.session_encryption_key} type="password" autocomplete="off" /></label>
		{/if}
		{#if telegramAuthMethod === 'bot_token'}
			<label class="wide"><span>{_('Bot token')}</span><input bind:value={telegramAccountForm.bot_token} type="password" autocomplete="off" /></label>
		{/if}
		<div class="wizard-note wide">{telegramWizardNote}</div>
		<div class="form-actions wide">
			{#if telegramAuthMethod === 'qr'}
				<button type="button" class="secondary-action" onclick={closeAccountDrawer}>{_('Cancel QR login')}</button>
				{#if telegramQrLogin?.status === 'waiting_password'}
					<button type="submit" disabled={isTelegramActionSubmitting || !telegramQrPassword}>{_('Continue')}</button>
				{/if}
				{#if telegramQrLogin?.status === 'ready'}
					<button type="button" onclick={() => void saveTelegramAccountFromWizard()} disabled={isTelegramActionSubmitting}>{_('Save Account')}</button>
				{/if}
			{:else}
				<button type="submit" disabled={isTelegramActionSubmitting}>{telegramAuthMethod === 'fixture' ? _('Save Fixture') : _('Save Live Record')}</button>
			{/if}
		</div>
	</form>
{/if}
