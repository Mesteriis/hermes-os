<script lang="ts">
	import { onMount } from 'svelte';
	import { apiBaseUrl } from '$lib/config';
	import { startGmailOAuthSetup, type GmailOAuthStartResponse } from '$lib/api';
	import {
		createGoogleWorkspaceOAuthStartRequest,
		saveImapAccount as saveImapAccountService,
		selectMailService as selectMailServiceModel,
		type MailService,
		type MailWizardStep,
		type Provider
	} from '$lib/services/accounts';
	import { buildGmailOAuthReturnUrl } from '$lib/services/oauth-callback';
	import { currentLocale, t } from '$lib/i18n';
	import MailAccountWizard from './MailAccountWizard.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		initialMailService: MailService;
		onAccountSaved?: () => Promise<void>;
	}

	let { initialMailService, onAccountSaved }: Props = $props();

	let selectedProvider = $state<Provider>('gmail');
	let mailWizardStep = $state<MailWizardStep>('provider');
	let selectedMailService = $state<MailService>('icloud');
	let isSetupSubmitting = $state(false);
	let setupMessage = $state('');
	let setupError = $state('');
	let gmailPending = $state<GmailOAuthStartResponse | null>(null);
	let gmailForm = $state({
		account_id: 'gmail-primary',
		display_name: 'Google Workspace',
		external_account_id: '',
		client_id: '',
		client_secret: '',
		redirect_uri: `${apiBaseUrl.replace(/\/+$/, '')}/api/v1/email-accounts/gmail/oauth/callback`
	});
	let imapForm = $state({
		account_id: 'icloud-primary',
		display_name: 'Primary iCloud',
		external_account_id: '',
		host: 'imap.mail.me.com',
		port: 993,
		tls: true,
		mailbox: 'INBOX',
		username: '',
		password: '',
		secret_kind: 'app_password' as 'app_password' | 'password',
		smtp_host: 'smtp.mail.me.com',
		smtp_port: 587,
		smtp_tls: true,
		smtp_starttls: true,
		smtp_username: ''
	});

	onMount(() => {
		selectMailService(initialMailService);
	});

	async function reloadAfterSave() {
		if (onAccountSaved) {
			await onAccountSaved();
		}
	}

	function selectMailService(service: MailService) {
		const next = selectMailServiceModel(service, gmailForm, imapForm);
		selectedProvider = next.selectedProvider;
		selectedMailService = next.selectedMailService;
		gmailForm = next.gmailForm as typeof gmailForm;
		imapForm = next.imapForm as typeof imapForm;
		setupMessage = '';
		setupError = '';
	}

	function chooseMailService(service: MailService) {
		selectMailService(service);
		mailWizardStep = 'details';
		if (service === 'gmail') {
			void startGmailSetup({ openAuthorization: true });
		}
	}

	function navigateGoogleAuthorization(authorizationUrl: string) {
		if (typeof window !== 'undefined') {
			window.open(authorizationUrl, '_blank', 'noopener,noreferrer');
		}
	}

	function gmailSetupErrorMessage(error: unknown) {
		const message = error instanceof Error ? error.message : 'Gmail setup failed';
		if (message.includes('host vault is locked')) {
			return _(
				'Hermes Secure Vault is locked. Unlock the vault, then start Google mail connection again.'
			);
		}
		if (message.includes('host vault is not initialized')) {
			return _(
				'Hermes Secure Vault is not initialized. Create the vault, then start Google mail connection again.'
			);
		}
		if (message.includes('HERMES_GOOGLE_OAUTH_CLIENT_ID')) {
			return _(
				'Google OAuth client credentials are not configured. Add HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH or HERMES_GOOGLE_OAUTH_CLIENT_ID to docker/.env, then restart make dev.'
			);
		}
		return message;
	}

	async function startGmailSetup(options: { openAuthorization?: boolean } = {}) {
		isSetupSubmitting = true;
		setupMessage = '';
		setupError = '';
		try {
			const request = createGoogleWorkspaceOAuthStartRequest(gmailForm);
			if (typeof window !== 'undefined') {
				request.app_return_url = buildGmailOAuthReturnUrl(window.location.origin);
			}
			gmailPending = await startGmailOAuthSetup(request);
			if (options.openAuthorization) {
				navigateGoogleAuthorization(gmailPending.authorization_url);
			}
			setupMessage = 'Google authorization opened in browser. Complete consent to save the account.';
		} catch (error) {
			setupError = gmailSetupErrorMessage(error);
		} finally {
			isSetupSubmitting = false;
		}
	}

	async function saveImapAccount() {
		isSetupSubmitting = true;
		setupMessage = '';
		setupError = '';
		try {
			const result = await saveImapAccountService({ selectedProvider, selectedMailService, imapForm });
			setupMessage = result.message;
			setupError = result.error;
			if (result.success) {
				imapForm = { ...imapForm, password: '' };
				await reloadAfterSave();
			}
		} finally {
			isSetupSubmitting = false;
		}
	}
</script>

<MailAccountWizard
	bind:mailWizardStep
	bind:imapForm
	{selectedMailService}
	{gmailPending}
	{isSetupSubmitting}
	{chooseMailService}
	{startGmailSetup}
	{reloadAfterSave}
	{saveImapAccount}
/>
{#if setupMessage}<p class="setup-state success">{setupMessage}</p>{/if}
{#if setupError}<p class="setup-state error">{setupError}</p>{/if}
