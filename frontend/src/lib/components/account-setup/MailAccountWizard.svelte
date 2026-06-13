<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { GmailOAuthStartResponse } from '$lib/api';
	import {
		hasFixedMailServerPreset,
		mailServiceDisplayName,
		mailServiceIcon,
		type MailService,
		type MailWizardStep
	} from '$lib/services/accounts';

	const _ = (key: string) => t($currentLocale, key);

	type ImapForm = {
		account_id: string;
		display_name: string;
		external_account_id: string;
		host: string;
		port: number;
		tls: boolean;
		mailbox: string;
		username: string;
		password: string;
		secret_kind: 'app_password' | 'password';
		smtp_host: string;
		smtp_port: number;
		smtp_tls: boolean;
		smtp_starttls: boolean;
		smtp_username: string;
	};

	interface Props {
		mailWizardStep: MailWizardStep;
		selectedMailService: MailService;
		imapForm: ImapForm;
		gmailPending: GmailOAuthStartResponse | null;
		isSetupSubmitting: boolean;
		chooseMailService: (service: MailService) => void;
		startGmailSetup: (options?: { openAuthorization?: boolean }) => Promise<void>;
		reloadAfterSave: () => Promise<void>;
		saveImapAccount: () => Promise<void>;
	}

	let {
		mailWizardStep = $bindable(),
		selectedMailService,
		imapForm = $bindable(),
		gmailPending,
		isSetupSubmitting,
		chooseMailService,
		startGmailSetup,
		reloadAfterSave,
		saveImapAccount
	}: Props = $props();
</script>

<div class="wizard-progress" aria-label={_('Mail setup steps')}>
	<span class:active={mailWizardStep === 'provider'}>{_('1. Service')}</span>
	<span class:active={mailWizardStep === 'details'}>{_('2. Details')}</span>
</div>

{#if mailWizardStep === 'provider'}
	<div class="wizard-step">
		<div class="wizard-choice-list">
			<button type="button" class:active={selectedMailService === 'icloud'} onclick={() => chooseMailService('icloud')}><Icon icon="tabler:cloud" width="34" height="34" /><strong>{_('iCloud')}</strong></button>
			<button type="button" class:active={selectedMailService === 'microsoft'} onclick={() => chooseMailService('microsoft')}><Icon icon="tabler:brand-office" width="34" height="34" /><strong>{_('Microsoft 365 / Exchange Online')}</strong></button>
			<button type="button" class:active={selectedMailService === 'gmail'} onclick={() => chooseMailService('gmail')}><Icon icon="tabler:brand-gmail" width="34" height="34" /><strong>{_('Google')}</strong></button>
			<button type="button" class:active={selectedMailService === 'fastmail'} onclick={() => chooseMailService('fastmail')}><Icon icon="tabler:mail-fast" width="34" height="34" /><strong>{_('Fastmail')}</strong></button>
			<button type="button" class:active={selectedMailService === 'mailru'} onclick={() => chooseMailService('mailru')}><Icon icon="tabler:mail" width="34" height="34" /><strong>{_('Mail.ru')}</strong></button>
			<button type="button" class:active={selectedMailService === 'yandex'} onclick={() => chooseMailService('yandex')}><Icon icon="tabler:letter-y" width="34" height="34" /><strong>{_('Yandex Mail')}</strong></button>
			<button type="button" class:active={selectedMailService === 'proton'} onclick={() => chooseMailService('proton')}><Icon icon="tabler:shield-lock" width="34" height="34" /><strong>{_('Proton Bridge')}</strong></button>
			<button type="button" class:active={selectedMailService === 'yahoo'} onclick={() => chooseMailService('yahoo')}><Icon icon="tabler:mail" width="34" height="34" /><strong>{_('Yahoo')}</strong></button>
			<button type="button" class:active={selectedMailService === 'aol'} onclick={() => chooseMailService('aol')}><Icon icon="tabler:mail-bolt" width="34" height="34" /><strong>{_('AOL')}</strong></button>
			<button type="button" class:active={selectedMailService === 'exchange'} onclick={() => chooseMailService('exchange')}><Icon icon="tabler:server-cog" width="34" height="34" /><strong>{_('Exchange IMAP')}</strong></button>
			<button type="button" class:active={selectedMailService === 'imap'} onclick={() => chooseMailService('imap')}><Icon icon="tabler:server" width="34" height="34" /><strong>{_('Other Mail Account')}</strong></button>
		</div>
	</div>
{:else}
	<div class="wizard-step">
		<button type="button" class="wizard-back" onclick={() => (mailWizardStep = 'provider')}><Icon icon="tabler:arrow-left" width="15" height="15" />{_('Service')}</button>
		{#if selectedMailService === 'gmail'}
			<div class="setup-summary-card" aria-label={_('Selected mail service')}>
				<span class="round-icon cyan"><Icon icon="tabler:brand-gmail" width="18" height="18" /></span>
				<div>
					<strong>{_('Google')}</strong>
					<p>{_('Authorization opened in browser. Complete Google consent to save the account.')}</p>
				</div>
			</div>
			{#if gmailPending}
				<div class="oauth-box">
					<a href={gmailPending.authorization_url} target="_blank" rel="noreferrer">{_('Reopen Google consent')}</a>
					<button type="button" onclick={() => void reloadAfterSave()} disabled={isSetupSubmitting}>{_('Refresh accounts')}</button>
				</div>
			{:else}
				<div class="form-actions wide"><button type="button" onclick={() => void startGmailSetup({ openAuthorization: true })} disabled={isSetupSubmitting}>{_('Start OAuth')}</button></div>
			{/if}
		{:else if hasFixedMailServerPreset(selectedMailService)}
			<div class="setup-summary-card" aria-label={_('Selected mail service')}>
				<span class="round-icon cyan"><Icon icon={mailServiceIcon(selectedMailService)} width="18" height="18" /></span>
				<div><strong>{mailServiceDisplayName(selectedMailService)}</strong></div>
			</div>
			<form class="setup-form compact-form" onsubmit={(event) => event.preventDefault()}>
				<label><span>{_('Email address')}</span><input bind:value={imapForm.username} type="email" autocomplete="email" /></label>
				<label><span>{_('Password')}</span><input bind:value={imapForm.password} type="password" autocomplete="current-password" /></label>
				<div class="form-actions wide"><button type="button" onclick={() => void saveImapAccount()} disabled={isSetupSubmitting}>{_('Save Account')}</button></div>
			</form>
		{:else}
			<form class="setup-form" onsubmit={(event) => event.preventDefault()}>
				<label><span>{_('Account ID')}</span><input bind:value={imapForm.account_id} autocomplete="off" /></label>
				<label><span>{_('Display name')}</span><input bind:value={imapForm.display_name} autocomplete="off" /></label>
				<label><span>{_('Email address')}</span><input bind:value={imapForm.external_account_id} autocomplete="email" /></label>
				<label><span>{_('Username')}</span><input bind:value={imapForm.username} autocomplete="username" /></label>
				<label><span>{_('Host')}</span><input bind:value={imapForm.host} autocomplete="off" /></label>
				<label><span>{_('Port')}</span><input bind:value={imapForm.port} type="number" min="1" max="65535" /></label>
				<label><span>{_('SMTP host')}</span><input bind:value={imapForm.smtp_host} autocomplete="off" /></label>
				<label><span>{_('SMTP port')}</span><input bind:value={imapForm.smtp_port} type="number" min="1" max="65535" /></label>
				<label><span>{_('SMTP username')}</span><input bind:value={imapForm.smtp_username} autocomplete="username" /></label>
				<label><span>{_('Mailbox')}</span><input bind:value={imapForm.mailbox} autocomplete="off" /></label>
				<label><span>{_('Password')}</span><input bind:value={imapForm.password} type="password" autocomplete="current-password" /></label>
				<label class="checkbox-row"><input bind:checked={imapForm.tls} type="checkbox" /><span>{_('TLS')}</span></label>
				<label class="checkbox-row"><input bind:checked={imapForm.smtp_tls} type="checkbox" /><span>{_('SMTP TLS')}</span></label>
				<label class="checkbox-row"><input bind:checked={imapForm.smtp_starttls} type="checkbox" /><span>{_('SMTP STARTTLS')}</span></label>
				<div class="form-actions"><button type="button" onclick={() => void saveImapAccount()} disabled={isSetupSubmitting}>{_('Save Account')}</button></div>
			</form>
		{/if}
	</div>
{/if}
