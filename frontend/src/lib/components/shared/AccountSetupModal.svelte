<script lang="ts">
	import { onDestroy } from 'svelte';
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import { accountWizardTarget, type AccountWizardKind } from '$lib/stores/accountWizard';
	import { apiBaseUrl } from '$lib/config';
	import {
		startGmailOAuthSetup,
		setupImapAccount,
		createCalendarAccount,
		setupTelegramAccount,
		setupTelegramFixtureAccount,
		setupWhatsappWebFixtureAccount,
		startTelegramQrLogin,
		submitTelegramQrLoginPassword,
		fetchTelegramQrLoginStatus,
		cancelTelegramQrLogin,
		fetchTelegramCapabilities,
		type GmailOAuthStartResponse,
		type TelegramQrLoginStatusResponse,
		type TelegramCapabilitiesResponse,
		type TelegramAccountSetupResponse,
		type TelegramLiveAccountSetupRequest,
		type TelegramProviderKind
	} from '$lib/api';
	import {
		hasFixedMailServerPreset,
		createGoogleWorkspaceOAuthStartRequest,
		mailServiceDisplayName,
		mailServiceIcon,
		mailServicePreset,
		mailServiceAccountPrefix,
		accountIdFromEmail,
		calendarProviderDefaultName,
		createTelegramAccountDraft,
		providerKindLabel
	} from '$lib/services/accounts';
	import { buildGmailOAuthReturnUrl } from '$lib/services/oauth-callback';
	import { shouldPollTelegramQrLoginStatus } from '$lib/services/telegram';

	const _ = (key: string) => t($currentLocale, key);

	type Provider = 'gmail' | 'icloud' | 'imap';
	type MailService = Provider | 'microsoft' | 'yahoo' | 'aol';
	type MailWizardStep = 'provider' | 'details';
	type CalendarProvider = 'local' | 'google' | 'microsoft' | 'apple' | 'caldav' | 'ics';
	type CalendarWizardStep = 'provider' | 'details';
	type TelegramSetupMode = 'fixture' | 'live';
	type TelegramAuthMethod = 'fixture' | 'phone' | 'qr' | 'bot_token';
	type TelegramWizardStep = 'account' | 'auth' | 'details';

	interface Props {
		isOpen: boolean;
		telegramCapabilities: TelegramCapabilitiesResponse | null;
		onAccountSaved?: () => Promise<void>;
	}

	let { isOpen = $bindable(), telegramCapabilities, onAccountSaved }: Props = $props();

	let selectedProvider = $state<Provider>('gmail');
	let accountWizardKind = $state<AccountWizardKind>('mail');
	let mailWizardStep = $state<MailWizardStep>('provider');
	let selectedMailService = $state<MailService>('icloud');
	let calendarWizardStep = $state<CalendarWizardStep>('provider');
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
		secret_kind: 'app_password' as 'app_password' | 'password'
	});
	let calendarAccountForm = $state({
		provider: 'local' as CalendarProvider,
		account_name: 'Local Calendar',
		email: ''
	});
	let telegramSetupMode = $state<TelegramSetupMode>('fixture');
	let telegramError = $state('');
	let telegramActionMessage = $state('');
	let isTelegramActionSubmitting = $state(false);
	let isTelegramCapabilitiesLoading = $state(false);
	let telegramAuthMethod = $state<TelegramAuthMethod>('fixture');
	let telegramWizardStep = $state<TelegramWizardStep>('account');
	let telegramQrLogin = $state<TelegramQrLoginStatusResponse | null>(null);
	let telegramQrPassword = $state('');
	let telegramQrAutoStartKey = $state('');
	let fetchedTelegramCapabilities = $state<TelegramCapabilitiesResponse | null>(null);
	let telegramCapabilitiesRequest: Promise<TelegramCapabilitiesResponse | null> | null = null;
	let telegramQrStatusPollTimer: ReturnType<typeof setTimeout> | null = null;
	let telegramQrStatusRequestInFlight = false;
	let telegramQrSessionGeneration = 0;
	let telegramQrReleasePromise: Promise<void> | null = null;
	let wasAccountDrawerOpen = false;
	let telegramAccountForm = $state(createTelegramAccountDraft());
	let whatsappAccountForm = $state({
		account_id: 'whatsapp-primary',
		display_name: 'Primary WhatsApp Web',
		external_account_id: 'whatsapp-fixture-device',
		device_name: 'Hermes Desktop Fixture',
		local_state_path: 'docker/data/whatsapp/whatsapp-primary'
	});
	let whatsappActionMessage = $state('');
	let whatsappError = $state('');
	let isWhatsappActionSubmitting = $state(false);

	const effectiveTelegramCapabilities = $derived(telegramCapabilities ?? fetchedTelegramCapabilities);
	const telegramQrRuntimeBlocked = $derived(
		telegramAuthMethod === 'qr' &&
			effectiveTelegramCapabilities !== null &&
			!effectiveTelegramCapabilities.tdjson_runtime_available
	);
	const telegramQrAppCredentialsMissing = $derived(
		telegramAuthMethod === 'qr' &&
			effectiveTelegramCapabilities !== null &&
			!effectiveTelegramCapabilities.telegram_app_credentials_configured
	);
	const telegramNeedsFormAppCredentials = $derived(telegramAuthMethod === 'phone');
	const telegramQrIsPreparing = $derived(
		telegramAuthMethod === 'qr' &&
			((!telegramQrLogin && (isTelegramCapabilitiesLoading || isTelegramActionSubmitting)) ||
				(telegramQrLogin?.status === 'waiting_qr_scan' && !telegramQrLogin.qr_svg))
	);

	$effect(() => {
		if (!isOpen) {
			if (wasAccountDrawerOpen) {
				releaseTelegramQrSession();
			}
			wasAccountDrawerOpen = false;
			return;
		}
		const isOpeningDrawer = !wasAccountDrawerOpen;
		wasAccountDrawerOpen = true;

		const target = $accountWizardTarget;
		accountWizardKind = target === 'gmail' || target === 'icloud' || target === 'imap' ? 'mail' : target;
		if (target === 'gmail' || target === 'icloud' || target === 'imap') {
			selectMailService(target);
		}
		if (accountWizardKind === 'mail') {
			mailWizardStep = 'provider';
		}
		if (accountWizardKind === 'calendar') {
			calendarWizardStep = 'provider';
		}
		if (accountWizardKind === 'telegram') {
			telegramWizardStep = 'account';
			if (isOpeningDrawer) {
				telegramAuthMethod = 'fixture';
				telegramSetupMode = 'fixture';
				telegramAccountForm = createTelegramAccountDraft();
			}
		}
		resetTelegramQrSessionLocal();
		setupMessage = '';
		setupError = '';
		telegramActionMessage = '';
		telegramError = '';
		whatsappActionMessage = '';
		whatsappError = '';
	});

	$effect(() => {
		const shouldAutoStartQrLogin =
			isOpen &&
			accountWizardKind === 'telegram' &&
			telegramWizardStep === 'details' &&
			telegramAuthMethod === 'qr' &&
			!telegramQrLogin &&
			!isTelegramActionSubmitting &&
			!setupError &&
			!telegramError;
		if (!shouldAutoStartQrLogin) {
			return;
		}

		const accountId = telegramAccountForm.account_id.trim() || 'telegram-primary';
		const autoStartKey = `${accountId}:${telegramWizardExternalAccountId()}`;
		if (telegramQrAutoStartKey === autoStartKey) {
			return;
		}

		telegramQrAutoStartKey = autoStartKey;
		void startTelegramQrLoginFromWizard();
	});

	$effect(() => {
		if (
			!isTelegramQrStepVisible() ||
			!shouldPollTelegramQrLoginStatus(telegramQrLogin?.status) ||
			isTelegramActionSubmitting ||
			telegramQrStatusRequestInFlight ||
			telegramQrStatusPollTimer
		) {
			return;
		}

		scheduleTelegramQrStatusPolling(telegramQrLogin);
	});

	onDestroy(() => {
		releaseTelegramQrSession();
	});

	function closeAccountDrawer() {
		releaseTelegramQrSession();
		isOpen = false;
	}

	async function reloadAfterSave() {
		if (onAccountSaved) {
			await onAccountSaved();
		}
	}

	function selectMailService(service: MailService) {
		selectedMailService = service;
		setupMessage = '';
		setupError = '';

		if (service === 'gmail') {
			selectedProvider = 'gmail';
			gmailForm = {
				...gmailForm,
				account_id: gmailForm.account_id || 'gmail-primary',
				display_name: gmailForm.display_name || 'Primary Gmail'
			};
			return;
		}

		const preset = mailServicePreset(service, imapForm);
		selectedProvider = preset.provider;
		imapForm = {
			...imapForm,
			account_id: preset.accountId,
			display_name: preset.displayName,
			host: preset.host,
			port: preset.port,
			tls: true,
			mailbox: imapForm.mailbox || 'INBOX',
			secret_kind: preset.secretKind
		};
	}

	function chooseMailService(service: MailService) {
		selectMailService(service);
		if (service === 'gmail') {
			continueMailWizard();
			void startGmailSetup({ openAuthorization: true });
			return;
		}
		continueMailWizard();
	}

	function continueMailWizard() {
		mailWizardStep = 'details';
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

	function continueTelegramWizard(nextStep: TelegramWizardStep) {
		const shouldPreserveQrSession =
			telegramAuthMethod === 'qr' &&
			nextStep === 'auth' &&
			isReusableTelegramQrLogin(telegramQrLogin);

		if (nextStep !== 'details') {
			if (!shouldPreserveQrSession) {
				releaseTelegramQrSession();
			} else {
				clearTelegramQrStatusPolling();
			}
		}
		telegramWizardStep = nextStep;

		if (
			nextStep === 'details' &&
			telegramAuthMethod === 'qr' &&
			telegramQrLogin?.status === 'waiting_qr_scan'
		) {
			scheduleTelegramQrStatusPolling(telegramQrLogin);
		}
	}

	function selectTelegramAuthMethod(method: TelegramAuthMethod) {
		const shouldPreserveQrSession =
			method === 'qr' &&
			telegramAuthMethod === 'qr' &&
			isReusableTelegramQrLogin(telegramQrLogin);

		telegramAuthMethod = method;
		telegramSetupMode = method === 'fixture' ? 'fixture' : 'live';
		if (!shouldPreserveQrSession) {
			releaseTelegramQrSession();
		} else {
			clearTelegramQrStatusPolling();
		}
		telegramError = '';
		telegramActionMessage = '';
		if (method === 'qr' && telegramAccountForm.external_account_id === '@telegram_fixture') {
			telegramAccountForm = {
				...telegramAccountForm,
				external_account_id: ''
			};
		} else if (method === 'bot_token') {
			telegramAccountForm = {
				...telegramAccountForm,
				provider_kind: 'telegram_bot'
			};
		}
	}

	function selectTelegramProviderKind(providerKind: TelegramProviderKind) {
		if (telegramAccountForm.provider_kind !== providerKind) {
			telegramAccountForm = createTelegramAccountDraft(providerKind);
			return;
		}

		telegramAccountForm = {
			...telegramAccountForm,
			provider_kind: providerKind
		};
	}

	function telegramWizardExternalAccountId() {
		return (
			telegramAccountForm.external_account_id.trim() ||
			(telegramAuthMethod === 'qr'
				? `qr-login:${telegramAccountForm.account_id}`
				: telegramAccountForm.account_id)
		);
	}

	function isReusableTelegramQrLogin(result: TelegramQrLoginStatusResponse | null) {
		return (
			result?.status === 'waiting_qr_scan' ||
			result?.status === 'waiting_password' ||
			result?.status === 'ready'
		);
	}

	function isTelegramQrStepVisible() {
		return (
			isOpen &&
			accountWizardKind === 'telegram' &&
			telegramWizardStep === 'details' &&
			telegramAuthMethod === 'qr'
		);
	}

	function resetTelegramQrSessionLocal(options: { resetAutoStartKey?: boolean } = {}) {
		clearTelegramQrStatusPolling();
		telegramQrLogin = null;
		telegramQrPassword = '';
		if (options.resetAutoStartKey !== false) {
			telegramQrAutoStartKey = '';
		}
	}

	function cancelTelegramQrBackendSession(setupId: string): Promise<void> {
		let releasePromise: Promise<void>;
		releasePromise = cancelTelegramQrLogin(setupId).catch(() => {
			// The session may already be closed or expired by the TDLib worker.
		}).then(() => undefined).finally(() => {
			if (telegramQrReleasePromise === releasePromise) {
				telegramQrReleasePromise = null;
			}
		});
		telegramQrReleasePromise = releasePromise;
		return releasePromise;
	}

	async function waitForTelegramQrSessionRelease() {
		const pendingRelease = telegramQrReleasePromise;
		if (pendingRelease) {
			await pendingRelease;
		}
	}

	function releaseTelegramQrSession(options: { cancelBackend?: boolean; resetAutoStartKey?: boolean } = {}) {
		const setupId = telegramQrLogin?.setup_id;
		const shouldCancel =
			options.cancelBackend !== false &&
			Boolean(setupId) &&
			isReusableTelegramQrLogin(telegramQrLogin);

		telegramQrSessionGeneration += 1;
		resetTelegramQrSessionLocal({ resetAutoStartKey: options.resetAutoStartKey });

		if (shouldCancel && setupId) {
			void cancelTelegramQrBackendSession(setupId);
		}
	}

	function telegramQrStatusLabel(status: string) {
		switch (status) {
			case 'waiting_qr_scan':
				return _('Waiting for QR scan');
			case 'waiting_password':
				return _('Telegram password required');
			case 'ready':
				return _('Telegram authorized');
			case 'expired':
				return _('QR code expired');
			case 'failed':
				return _('QR login failed');
			case 'runtime_unavailable':
				return _('TDLib runtime unavailable');
			default:
				return _('QR login status');
		}
	}

	function clearTelegramQrStatusPolling() {
		if (!telegramQrStatusPollTimer) {
			return;
		}
		clearTimeout(telegramQrStatusPollTimer);
		telegramQrStatusPollTimer = null;
	}

	function scheduleTelegramQrStatusPolling(result: TelegramQrLoginStatusResponse | null) {
		clearTelegramQrStatusPolling();
		if (!isTelegramQrStepVisible() || !shouldPollTelegramQrLoginStatus(result?.status)) {
			return;
		}
		const delay = Math.max(1000, Math.min(result.poll_after_ms || 2500, 10_000));
		telegramQrStatusPollTimer = setTimeout(() => {
			telegramQrStatusPollTimer = null;
			void refreshTelegramQrLoginStatus({ silent: true });
		}, delay);
	}

	async function ensureTelegramCapabilities(): Promise<TelegramCapabilitiesResponse | null> {
		const currentCapabilities = telegramCapabilities ?? fetchedTelegramCapabilities;
		if (currentCapabilities) {
			return currentCapabilities;
		}
		if (!telegramCapabilitiesRequest) {
			isTelegramCapabilitiesLoading = true;
			telegramCapabilitiesRequest = fetchTelegramCapabilities()
				.then((capabilities) => {
					fetchedTelegramCapabilities = capabilities;
					return capabilities;
				})
				.catch(() => null)
				.finally(() => {
					isTelegramCapabilitiesLoading = false;
					telegramCapabilitiesRequest = null;
				});
		}
		return telegramCapabilitiesRequest;
	}

	function applyTelegramQrLoginResult(result: TelegramQrLoginStatusResponse) {
		telegramQrLogin = result;
		if (result.status !== 'ready') {
			return;
		}
		telegramQrPassword = '';
		telegramAccountForm = {
			...telegramAccountForm,
			account_id: result.suggested_account_id ?? telegramAccountForm.account_id,
			display_name: result.suggested_display_name ?? telegramAccountForm.display_name,
			external_account_id:
				result.suggested_external_account_id ?? telegramAccountForm.external_account_id
		};
	}

	function telegramWizardNote() {
		if (telegramAuthMethod === 'fixture') {
			return _('Fixture mode creates local Telegram records for UI and policy testing.');
		}
		if (telegramAuthMethod === 'qr') {
			if (isTelegramCapabilitiesLoading && !effectiveTelegramCapabilities) {
				return _('Preparing Telegram QR login...');
			}
			if (telegramQrRuntimeBlocked) {
				return _('TDLib JSON runtime is not available in the running backend.');
			}
			if (telegramQrAppCredentialsMissing) {
				return _('Telegram app credentials must be configured in the backend environment before QR login.');
			}
			if (telegramQrLogin?.status === 'waiting_password') {
				return _('Enter the Telegram 2-step verification password to finish local TDLib authorization.');
			}
			if (telegramQrLogin?.status === 'ready') {
				return _('Telegram authorization is complete. Save the account to finish setup.');
			}
			return _('Open Telegram on an already logged-in device and scan the QR code.');
		}
		return _('Live credentials are stored in the encrypted database vault. Telegram live runtime remains blocked until the adapter is implemented.');
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
			const username = imapForm.username.trim();
			const fixedServerPreset = hasFixedMailServerPreset(selectedMailService);
			const externalAccountId = fixedServerPreset
				? username
				: imapForm.external_account_id.trim() || username;
			const accountId = fixedServerPreset
				? accountIdFromEmail(externalAccountId, mailServiceAccountPrefix(selectedMailService))
				: imapForm.account_id.trim() || accountIdFromEmail(externalAccountId, 'imap');
			const displayName = fixedServerPreset
				? externalAccountId || mailServiceDisplayName(selectedMailService)
				: imapForm.display_name.trim() || externalAccountId || mailServiceDisplayName(selectedMailService);
			const host = imapForm.host.trim();
			const mailbox = imapForm.mailbox.trim() || 'INBOX';

			const result = await setupImapAccount({
				account_id: accountId,
				provider_kind: selectedProvider === 'icloud' ? 'icloud' : 'imap',
				display_name: displayName,
				external_account_id: externalAccountId,
				host,
				port: Number(imapForm.port),
				tls: imapForm.tls,
				mailbox,
				username,
				password: imapForm.password,
				secret_kind: imapForm.secret_kind
			});
			setupMessage = `Mail account ${result.account_id} saved`;
			imapForm = { ...imapForm, password: '' };
			await reloadAfterSave();
		} catch (error) {
			setupError = error instanceof Error ? error.message : 'Mail account setup failed';
		} finally {
			isSetupSubmitting = false;
		}
	}

	async function saveCalendarAccount() {
		isSetupSubmitting = true;
		setupMessage = '';
		setupError = '';

		try {
			const result = await createCalendarAccount({
				provider: calendarAccountForm.provider,
				account_name: calendarAccountForm.account_name,
				email: calendarAccountForm.email.trim() || undefined
			});
			setupMessage = `Calendar account ${result.account_name} saved`;
			await reloadAfterSave();
		} catch (error) {
			setupError = error instanceof Error ? error.message : 'Calendar account setup failed';
		} finally {
			isSetupSubmitting = false;
		}
	}

	async function setupWhatsappWebFixture() {
		if (isWhatsappActionSubmitting) {
			return;
		}

		isWhatsappActionSubmitting = true;
		whatsappActionMessage = '';
		whatsappError = '';
		setupMessage = '';
		setupError = '';
		try {
			const result = await setupWhatsappWebFixtureAccount({
				account_id: whatsappAccountForm.account_id,
				provider_kind: 'whatsapp_web',
				display_name: whatsappAccountForm.display_name,
				external_account_id: whatsappAccountForm.external_account_id,
				device_name: whatsappAccountForm.device_name,
				local_state_path: whatsappAccountForm.local_state_path
			});
			whatsappActionMessage = `${providerKindLabel(result.provider_kind)} account ${result.account_id} saved`;
			setupMessage = whatsappActionMessage;
			await reloadAfterSave();
		} catch (error) {
			const message = error instanceof Error ? error.message : 'WhatsApp Web fixture setup failed';
			whatsappError = message;
			setupError = message;
		} finally {
			isWhatsappActionSubmitting = false;
		}
	}

	async function saveTelegramAccountFromWizard(options: { allowWhileSubmitting?: boolean } = {}) {
		if (isTelegramActionSubmitting && !options.allowWhileSubmitting) {
			return;
		}
		const shouldResetSubmitting = !isTelegramActionSubmitting;

		const isFixtureSetup = telegramAuthMethod === 'fixture';
		const providerKind =
			telegramAuthMethod === 'bot_token'
				? 'telegram_bot'
				: telegramAuthMethod === 'phone' || telegramAuthMethod === 'qr'
					? 'telegram_user'
					: telegramAccountForm.provider_kind;
		const externalAccountId = telegramWizardExternalAccountId();
		const isQrAuthorizedAccount =
			telegramAuthMethod === 'qr' && telegramQrLogin?.status === 'ready';

		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		setupMessage = '';
		setupError = '';
		try {
			let result: TelegramAccountSetupResponse;
			if (isFixtureSetup) {
				result = await setupTelegramFixtureAccount({
					account_id: telegramAccountForm.account_id,
					provider_kind: providerKind,
					display_name: telegramAccountForm.display_name,
					external_account_id: externalAccountId,
					tdlib_data_path: telegramAccountForm.tdlib_data_path || undefined,
					transcription_enabled:
						telegramAuthMethod === 'qr' ? false : telegramAccountForm.transcription_enabled
				});
			} else {
				const request: TelegramLiveAccountSetupRequest = {
					account_id: telegramAccountForm.account_id,
					provider_kind: providerKind,
					display_name: telegramAccountForm.display_name,
					external_account_id: externalAccountId,
					tdlib_data_path: telegramAccountForm.tdlib_data_path || undefined,
					transcription_enabled:
						telegramAuthMethod === 'qr' ? false : telegramAccountForm.transcription_enabled
				};
				if (providerKind === 'telegram_user' && isQrAuthorizedAccount) {
					request.qr_authorized = true;
				} else if (providerKind === 'telegram_user') {
					const apiId = telegramAccountForm.api_id.trim();
					const apiHash = telegramAccountForm.api_hash.trim();
					if (apiId) {
						request.api_id = Number(apiId);
					}
					if (apiHash) {
						request.api_hash = apiHash;
					}
					if (telegramAccountForm.session_encryption_key) {
						request.session_encryption_key = telegramAccountForm.session_encryption_key;
					}
				} else if (providerKind === 'telegram_bot' && telegramAccountForm.bot_token) {
					request.bot_token = telegramAccountForm.bot_token;
				}
				result = await setupTelegramAccount(request);
			}
			const runtimeLabel =
				telegramAuthMethod === 'qr' && telegramQrLogin?.status === 'ready'
					? 'saved after QR authorization'
					: result.runtime === 'live_blocked'
						? 'saved as live-blocked'
						: 'saved';
			telegramActionMessage = `${providerKindLabel(result.provider_kind)} account ${result.account_id} ${runtimeLabel}`;
			setupMessage = telegramActionMessage;
			telegramAccountForm = {
				...telegramAccountForm,
				api_hash: '',
				bot_token: '',
				session_encryption_key: ''
			};
			await reloadAfterSave();
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Telegram account setup failed';
			setupError = message;
		} finally {
			if (shouldResetSubmitting) {
				isTelegramActionSubmitting = false;
			}
		}
	}

	async function saveReadyTelegramQrAccountFromWizard() {
		if (telegramAuthMethod !== 'qr' || telegramQrLogin?.status !== 'ready') {
			return;
		}
		await saveTelegramAccountFromWizard({ allowWhileSubmitting: true });
	}

	async function startTelegramQrLoginFromWizard() {
		if (isTelegramActionSubmitting) {
			return;
		}

		releaseTelegramQrSession({ resetAutoStartKey: false });
		const sessionGeneration = telegramQrSessionGeneration;
		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		setupMessage = '';
		setupError = '';

		try {
			await waitForTelegramQrSessionRelease();
			if (sessionGeneration !== telegramQrSessionGeneration || !isTelegramQrStepVisible()) {
				return;
			}
			const capabilities = await ensureTelegramCapabilities();
			if (sessionGeneration !== telegramQrSessionGeneration || !isTelegramQrStepVisible()) {
				return;
			}
			if (!capabilities) {
				throw new Error('Telegram backend capabilities could not be loaded');
			}
			if (!capabilities.tdjson_runtime_available) {
				throw new Error('TDLib JSON runtime is not available in the running backend');
			}
			if (!capabilities.telegram_app_credentials_configured) {
				throw new Error('Telegram app credentials are not configured in the backend environment');
			}

			const result = await startTelegramQrLogin({
				account_id: telegramAccountForm.account_id,
				display_name: telegramAccountForm.display_name,
				external_account_id: telegramWizardExternalAccountId(),
				tdlib_data_path: telegramAccountForm.tdlib_data_path || undefined,
				transcription_enabled: false
			});
			if (sessionGeneration !== telegramQrSessionGeneration || !isTelegramQrStepVisible()) {
				void cancelTelegramQrBackendSession(result.setup_id);
				return;
			}
			applyTelegramQrLoginResult(result);
			if (result.status === 'ready') {
				await saveReadyTelegramQrAccountFromWizard();
			} else {
				scheduleTelegramQrStatusPolling(result);
				setupMessage =
					result.status === 'waiting_qr_scan' && result.qr_svg
						? 'Scan the Telegram QR code to continue'
						: result.message ?? `Telegram QR login status: ${result.status}`;
			}
		} catch (error) {
			if (sessionGeneration !== telegramQrSessionGeneration || !isTelegramQrStepVisible()) {
				return;
			}
			const message = error instanceof Error ? error.message : 'Telegram QR login start failed';
			setupError = message;
			telegramError = message;
		} finally {
			isTelegramActionSubmitting = false;
		}
	}

	async function submitTelegramQrPasswordFromWizard() {
		if (isTelegramActionSubmitting || !telegramQrLogin) {
			return;
		}

		if (telegramQrLogin.status !== 'waiting_password') {
			setupError = 'Telegram QR login is not waiting for a password';
			telegramError = setupError;
			return;
		}

		if (!telegramQrPassword) {
			setupError = 'Telegram 2-step verification password is required';
			telegramError = setupError;
			return;
		}

		const setupId = telegramQrLogin.setup_id;
		const sessionGeneration = telegramQrSessionGeneration;
		isTelegramActionSubmitting = true;
		setupError = '';
		telegramError = '';
		try {
			const result = await submitTelegramQrLoginPassword(
				setupId,
				{ password: telegramQrPassword }
			);
			if (
				sessionGeneration !== telegramQrSessionGeneration ||
				telegramQrLogin?.setup_id !== setupId ||
				!isTelegramQrStepVisible()
			) {
				return;
			}
			telegramQrPassword = '';
			applyTelegramQrLoginResult(result);
			if (result.status === 'ready') {
				await saveReadyTelegramQrAccountFromWizard();
			} else {
				scheduleTelegramQrStatusPolling(result);
				setupMessage = result.message ?? `Telegram QR login status: ${result.status}`;
			}
		} catch (error) {
			if (sessionGeneration !== telegramQrSessionGeneration || !isTelegramQrStepVisible()) {
				return;
			}
			const message =
				error instanceof Error ? error.message : 'Telegram QR login password submit failed';
			setupError = message;
			telegramError = message;
		} finally {
			isTelegramActionSubmitting = false;
		}
	}

	function submitTelegramQrStepFromWizard() {
		if (telegramQrLogin?.status === 'waiting_password') {
			void submitTelegramQrPasswordFromWizard();
			return;
		}
		void startTelegramQrLoginFromWizard();
	}

	async function refreshTelegramQrLoginStatus(options: { silent?: boolean } = {}) {
		if (!telegramQrLogin || isTelegramActionSubmitting || telegramQrStatusRequestInFlight) {
			return;
		}

		const setupId = telegramQrLogin.setup_id;
		const sessionGeneration = telegramQrSessionGeneration;
		const hadQrSvg = Boolean(telegramQrLogin.qr_svg);
		telegramQrStatusRequestInFlight = true;
		if (!options.silent) {
			isTelegramActionSubmitting = true;
			setupError = '';
			telegramError = '';
		}
		try {
			const result = await fetchTelegramQrLoginStatus(setupId);
			if (
				sessionGeneration !== telegramQrSessionGeneration ||
				telegramQrLogin?.setup_id !== setupId ||
				!isTelegramQrStepVisible()
			) {
				return;
			}
			if (
				options.silent &&
				telegramQrLogin.status === 'waiting_qr_scan' &&
				result.status === 'waiting_qr_scan' &&
				Boolean(telegramQrLogin.qr_svg) &&
				Boolean(result.qr_svg)
			) {
				scheduleTelegramQrStatusPolling(result);
				return;
			}

			applyTelegramQrLoginResult(result);
			if (result.status === 'ready') {
				await saveReadyTelegramQrAccountFromWizard();
			} else {
				scheduleTelegramQrStatusPolling(result);
				const didReceiveFirstQrSvg =
					result.status === 'waiting_qr_scan' && !hadQrSvg && Boolean(result.qr_svg);
				if (!options.silent || result.status !== 'waiting_qr_scan' || didReceiveFirstQrSvg) {
					setupMessage =
						didReceiveFirstQrSvg
							? 'Scan the Telegram QR code to continue'
							: result.message ?? `Telegram QR login status: ${result.status}`;
				}
			}
		} catch (error) {
			if (sessionGeneration !== telegramQrSessionGeneration || !isTelegramQrStepVisible()) {
				return;
			}
			const message =
				error instanceof Error ? error.message : 'Telegram QR login status request failed';
			setupError = message;
			telegramError = message;
			clearTelegramQrStatusPolling();
		} finally {
			telegramQrStatusRequestInFlight = false;
			if (!options.silent) {
				isTelegramActionSubmitting = false;
			}
		}
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
				<p>{accountWizardKind === 'mail' ? _('Mail Account') : accountWizardKind === 'calendar' ? _('Calendar Account') : accountWizardKind === 'telegram' ? _('Telegram Account') : _('WhatsApp Account')}</p>
				<h2 id="account-setup-heading">{accountWizardKind === 'mail' ? _('New Mail Account') : accountWizardKind === 'calendar' ? _('New Calendar Account') : accountWizardKind === 'telegram' ? _('New Telegram Account') : _('New WhatsApp Account')}</h2>
			</div>
			<button type="button" class="icon-button" onclick={closeAccountDrawer} aria-label={_('Close')}>
				<Icon icon="tabler:x" width="18" height="18" />
			</button>
		</header>

		{#if accountWizardKind === 'mail'}
			<div class="wizard-progress" aria-label={_('Mail setup steps')}>
				<span class:active={mailWizardStep === 'provider'}>{_('1. Service')}</span>
				<span class:active={mailWizardStep === 'details'}>{_('2. Details')}</span>
			</div>
			{#if mailWizardStep === 'provider'}
				<div class="wizard-step">
					<div class="wizard-choice-list">
						<button type="button" class:active={selectedMailService === 'icloud'} onclick={() => chooseMailService('icloud')}><Icon icon="tabler:cloud" width="34" height="34" /><strong>{_('iCloud')}</strong></button>
						<button type="button" class:active={selectedMailService === 'microsoft'} onclick={() => chooseMailService('microsoft')}><Icon icon="tabler:brand-office" width="34" height="34" /><strong>{_('Microsoft Exchange')}</strong></button>
						<button type="button" class:active={selectedMailService === 'gmail'} onclick={() => chooseMailService('gmail')}><Icon icon="tabler:brand-gmail" width="34" height="34" /><strong>{_('Google')}</strong></button>
						<button type="button" class:active={selectedMailService === 'yahoo'} onclick={() => chooseMailService('yahoo')}><Icon icon="tabler:mail" width="34" height="34" /><strong>{_('Yahoo')}</strong></button>
						<button type="button" class:active={selectedMailService === 'aol'} onclick={() => chooseMailService('aol')}><Icon icon="tabler:mail-bolt" width="34" height="34" /><strong>{_('AOL')}</strong></button>
						<button type="button" class:active={selectedMailService === 'imap'} onclick={() => chooseMailService('imap')}><Icon icon="tabler:server" width="34" height="34" /><strong>{_('Other Mail Account')}</strong></button>
					</div>
				</div>
			{:else}
				<div class="wizard-step">
					<button type="button" class="wizard-back" onclick={() => (mailWizardStep = 'provider' as MailWizardStep)}><Icon icon="tabler:arrow-left" width="15" height="15" />{_('Service')}</button>
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
							<div>
								<strong>{mailServiceDisplayName(selectedMailService)}</strong>
							</div>
						</div>
						<form class="setup-form compact-form" onsubmit={(event) => event.preventDefault()}>
							<label><span>{_('Email address')}</span><input bind:value={imapForm.username} type="email" autocomplete="email" /></label>
							<label><span>{_('Password')}</span><input bind:value={imapForm.password} type="password" autocomplete="current-password" /></label>
							<div class="form-actions wide"><button type="button" onclick={saveImapAccount} disabled={isSetupSubmitting}>{_('Save Account')}</button></div>
						</form>
					{:else}
						<form class="setup-form" onsubmit={(event) => event.preventDefault()}>
							<label><span>{_('Account ID')}</span><input bind:value={imapForm.account_id} autocomplete="off" /></label>
							<label><span>{_('Display name')}</span><input bind:value={imapForm.display_name} autocomplete="off" /></label>
							<label><span>{_('Email address')}</span><input bind:value={imapForm.external_account_id} autocomplete="email" /></label>
							<label><span>{_('Username')}</span><input bind:value={imapForm.username} autocomplete="username" /></label>
							<label><span>{_('Host')}</span><input bind:value={imapForm.host} autocomplete="off" /></label>
							<label><span>{_('Port')}</span><input bind:value={imapForm.port} type="number" min="1" max="65535" /></label>
							<label><span>{_('Mailbox')}</span><input bind:value={imapForm.mailbox} autocomplete="off" /></label>
							<label><span>{_('Password')}</span><input bind:value={imapForm.password} type="password" autocomplete="current-password" /></label>
							<label class="checkbox-row"><input bind:checked={imapForm.tls} type="checkbox" /><span>{_('TLS')}</span></label>
							<div class="form-actions"><button type="button" onclick={saveImapAccount} disabled={isSetupSubmitting}>{_('Save Account')}</button></div>
						</form>
					{/if}
				</div>
			{/if}
		{:else if accountWizardKind === 'calendar'}
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
					<button type="button" class="wizard-back wide" onclick={() => (calendarWizardStep = 'provider' as CalendarWizardStep)}><Icon icon="tabler:arrow-left" width="15" height="15" />{_('Provider')}</button>
					<label><span>{_('Provider')}</span><select bind:value={calendarAccountForm.provider}><option value="local">{_('Local')}</option><option value="google">{_('Google Calendar')}</option><option value="microsoft">{_('Microsoft 365')}</option><option value="apple">{_('Apple Calendar')}</option><option value="caldav">{_('CalDAV')}</option><option value="ics">{_('ICS Feed')}</option></select></label>
					<label><span>{_('Account name')}</span><input bind:value={calendarAccountForm.account_name} autocomplete="off" /></label>
					<label class="wide"><span>{_('Email or owner')}</span><input bind:value={calendarAccountForm.email} autocomplete="email" /></label>
					<div class="form-actions wide"><button type="submit" disabled={isSetupSubmitting || !calendarAccountForm.account_name.trim()}>{_('Save Calendar')}</button></div>
				</form>
			{/if}
		{:else if accountWizardKind === 'telegram'}
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
						<div class="qr-login-panel telegram-qr-panel wide" class:large={Boolean(telegramQrLogin?.qr_svg)}>
							{#if telegramQrLogin?.qr_svg}
								<div class="qr-svg" aria-label={_('Telegram QR code')}>{@html telegramQrLogin.qr_svg}</div>
							{:else if telegramQrIsPreparing}
								<div class="qr-skeleton" aria-label={_('Loading Telegram QR code')}>
									<span></span>
									<span></span>
									<span></span>
								</div>
							{:else}
								<Icon icon="tabler:qrcode" width="58" height="58" />
							{/if}
							<div class="qr-login-copy">
								<strong>{telegramQrLogin ? telegramQrStatusLabel(telegramQrLogin.status) : telegramQrIsPreparing ? _('Preparing QR login') : _('QR login')}</strong>
								<p>{telegramQrLogin?.message ?? (telegramQrIsPreparing ? _('Requesting a fresh Telegram QR code...') : _('Telegram QR login starts automatically for this account.'))}</p>
								<p class="qr-account-hint">{_('Account ID')}: {telegramAccountForm.account_id}</p>
								{#if telegramQrLogin?.qr_link}
									<a href={telegramQrLogin.qr_link}>{_('Open Telegram login link')}</a>
								{/if}
							</div>
						</div>
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
						<div class="wizard-note wide">
							{telegramWizardNote()}
						</div>
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
		{:else if accountWizardKind === 'whatsapp'}
			<form class="setup-form" onsubmit={(event) => { event.preventDefault(); void setupWhatsappWebFixture(); }}>
				<label><span>{_('Account ID')}</span><input bind:value={whatsappAccountForm.account_id} autocomplete="off" /></label>
				<label><span>{_('Display name')}</span><input bind:value={whatsappAccountForm.display_name} autocomplete="off" /></label>
				<label><span>{_('External ID')}</span><input bind:value={whatsappAccountForm.external_account_id} autocomplete="off" /></label>
				<label><span>{_('Device name')}</span><input bind:value={whatsappAccountForm.device_name} autocomplete="off" /></label>
				<label class="wide"><span>{_('Local state path')}</span><input bind:value={whatsappAccountForm.local_state_path} autocomplete="off" /></label>
				<div class="wizard-note wide">{_('WhatsApp Web live runtime remains blocked; this creates a fixture companion-session record.')}</div>
				<div class="form-actions wide"><button type="submit" disabled={isWhatsappActionSubmitting}>{_('Save Fixture')}</button></div>
			</form>
		{/if}

		{#if setupMessage}<p class="setup-state success">{setupMessage}</p>{/if}
		{#if setupError}<p class="setup-state error">{setupError}</p>{/if}
	</div>
{/if}
