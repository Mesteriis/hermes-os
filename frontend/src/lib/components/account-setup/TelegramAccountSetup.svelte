<script lang="ts">
	import './telegramQr.css';
	import { onDestroy } from 'svelte';
	import { cancelTelegramQrLogin, fetchTelegramCapabilities, type TelegramCapabilitiesResponse, type TelegramProviderKind, type TelegramQrLoginStatusResponse } from '$lib/api';
	import {
		createTelegramAccountDraft,
		telegramQrStatusLabel as telegramQrStatusLabelText,
		type TelegramAuthMethod,
		type TelegramSetupMode,
		type TelegramWizardStep
	} from '$lib/services/accounts';
	import {
		refreshTelegramQrLoginStatus as refreshTelegramQrLoginStatusService,
		saveReadyTelegramQrAccountFromWizard as shouldSaveReadyTelegramQrAccountFromWizard,
		saveTelegramAccountFromWizard as saveTelegramAccountFromWizardService,
		shouldPollTelegramQrLoginStatus,
		startTelegramQrLoginFromWizard as startTelegramQrLoginFromWizardService,
		submitTelegramQrPasswordFromWizard as submitTelegramQrPasswordFromWizardService,
		submitTelegramQrStepFromWizard as nextTelegramQrStepFromWizard
	} from '$lib/services/telegram';
	import { currentLocale, t } from '$lib/i18n';
	import TelegramAccountWizard from './TelegramAccountWizard.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		isOpen: boolean;
		telegramCapabilities: TelegramCapabilitiesResponse | null;
		closeAccountDrawer: () => void;
		onAccountSaved?: () => Promise<void>;
	}

	let { isOpen, telegramCapabilities, closeAccountDrawer, onAccountSaved }: Props = $props();

	let telegramSetupMode = $state<TelegramSetupMode>('fixture');
	let telegramError = $state('');
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
	let setupMessage = $state('');
	let setupError = $state('');
	let telegramAccountForm = $state(createTelegramAccountDraft());

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
		if (!isTelegramQrStepVisible() || telegramQrLogin || isTelegramActionSubmitting || setupError || telegramError) {
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

	async function reloadAfterSave() {
		if (onAccountSaved) {
			await onAccountSaved();
		}
	}

	function continueTelegramWizard(nextStep: TelegramWizardStep) {
		const preserveQr =
			telegramAuthMethod === 'qr' &&
			nextStep === 'auth' &&
			isReusableTelegramQrLogin(telegramQrLogin);
		if (nextStep !== 'details') {
			preserveQr ? clearTelegramQrStatusPolling() : releaseTelegramQrSession();
		}
		telegramWizardStep = nextStep;
		if (nextStep === 'details' && telegramAuthMethod === 'qr' && telegramQrLogin?.status === 'waiting_qr_scan') {
			scheduleTelegramQrStatusPolling(telegramQrLogin);
		}
	}

	function selectTelegramAuthMethod(method: TelegramAuthMethod) {
		const preserveQr =
			method === 'qr' && telegramAuthMethod === 'qr' && isReusableTelegramQrLogin(telegramQrLogin);
		telegramAuthMethod = method;
		telegramSetupMode = method === 'fixture' ? 'fixture' : 'live';
		preserveQr ? clearTelegramQrStatusPolling() : releaseTelegramQrSession();
		telegramError = '';
		setupError = '';
		if (method === 'qr' && telegramAccountForm.external_account_id === '@telegram_fixture') {
			telegramAccountForm = { ...telegramAccountForm, external_account_id: '' };
		} else if (method === 'bot_token') {
			telegramAccountForm = { ...telegramAccountForm, provider_kind: 'telegram_bot' };
		}
	}

	function selectTelegramProviderKind(providerKind: TelegramProviderKind) {
		telegramAccountForm =
			telegramAccountForm.provider_kind !== providerKind
				? createTelegramAccountDraft(providerKind)
				: { ...telegramAccountForm, provider_kind: providerKind };
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
		return result?.status === 'waiting_qr_scan' || result?.status === 'waiting_password' || result?.status === 'ready';
	}

	function isTelegramQrStepVisible() {
		return isOpen && telegramWizardStep === 'details' && telegramAuthMethod === 'qr';
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
		releasePromise = cancelTelegramQrLogin(setupId)
			.then(() => undefined)
			.catch(() => undefined)
			.finally(() => {
				if (telegramQrReleasePromise === releasePromise) {
					telegramQrReleasePromise = null;
				}
			});
		telegramQrReleasePromise = releasePromise;
		return releasePromise;
	}

	async function waitForTelegramQrSessionRelease() {
		if (telegramQrReleasePromise) {
			await telegramQrReleasePromise;
		}
	}

	function releaseTelegramQrSession(options: { cancelBackend?: boolean; resetAutoStartKey?: boolean } = {}) {
		const setupId = telegramQrLogin?.setup_id;
		const shouldCancel = options.cancelBackend !== false && Boolean(setupId) && isReusableTelegramQrLogin(telegramQrLogin);
		telegramQrSessionGeneration += 1;
		resetTelegramQrSessionLocal({ resetAutoStartKey: options.resetAutoStartKey });
		if (shouldCancel && setupId) {
			void cancelTelegramQrBackendSession(setupId);
		}
	}

	function telegramQrStatusLabel(status: string) {
		return _(telegramQrStatusLabelText(status));
	}

	function clearTelegramQrStatusPolling() {
		if (telegramQrStatusPollTimer) {
			clearTimeout(telegramQrStatusPollTimer);
			telegramQrStatusPollTimer = null;
		}
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
		if (effectiveTelegramCapabilities) {
			return effectiveTelegramCapabilities;
		}
		if (!telegramCapabilitiesRequest) {
			isTelegramCapabilitiesLoading = true;
			telegramCapabilitiesRequest = fetchTelegramCapabilities()
				.then((capabilities) => (fetchedTelegramCapabilities = capabilities))
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
		if (result.status === 'ready') {
			telegramQrPassword = '';
			telegramAccountForm = {
				...telegramAccountForm,
				account_id: result.suggested_account_id ?? telegramAccountForm.account_id,
				display_name: result.suggested_display_name ?? telegramAccountForm.display_name,
				external_account_id: result.suggested_external_account_id ?? telegramAccountForm.external_account_id
			};
		}
	}

	function telegramWizardNote() {
		if (telegramAuthMethod === 'fixture') {
			return _('Fixture mode creates local Telegram records for UI and policy testing.');
		}
		if (telegramAuthMethod !== 'qr') {
			return _('Live credentials are stored in the encrypted database vault. Telegram live runtime remains blocked until the adapter is implemented.');
		}
		if (isTelegramCapabilitiesLoading && !effectiveTelegramCapabilities) return _('Preparing Telegram QR login...');
		if (telegramQrRuntimeBlocked) return _('TDLib JSON runtime is not available in the running backend.');
		if (telegramQrAppCredentialsMissing) return _('Telegram app credentials must be configured in the backend environment before QR login.');
		if (telegramQrLogin?.status === 'waiting_password') return _('Enter the Telegram 2-step verification password to finish local TDLib authorization.');
		if (telegramQrLogin?.status === 'ready') return _('Telegram authorization is complete. Save the account to finish setup.');
		return _('Open Telegram on an already logged-in device and scan the QR code.');
	}

	async function saveTelegramAccountFromWizard(options: { allowWhileSubmitting?: boolean } = {}) {
		if (isTelegramActionSubmitting && !options.allowWhileSubmitting) {
			return;
		}
		const shouldResetSubmitting = !isTelegramActionSubmitting;
		isTelegramActionSubmitting = true;
		setupMessage = '';
		setupError = '';
		const result = await saveTelegramAccountFromWizardService({
			accountForm: { ...telegramAccountForm, external_account_id: telegramWizardExternalAccountId() },
			authMethod: telegramAuthMethod,
			qrLogin: telegramQrLogin,
			isFixtureSetup: telegramSetupMode === 'fixture'
		});
		setupMessage = result.message;
		setupError = result.error;
		if (!result.error) {
			telegramAccountForm = { ...telegramAccountForm, api_hash: '', bot_token: '', session_encryption_key: '' };
			await reloadAfterSave();
		}
		if (shouldResetSubmitting) {
			isTelegramActionSubmitting = false;
		}
	}

	async function saveReadyTelegramQrAccountFromWizard() {
		const result = await shouldSaveReadyTelegramQrAccountFromWizard(telegramAuthMethod, telegramQrLogin);
		if (result.shouldSave) {
			await saveTelegramAccountFromWizard({ allowWhileSubmitting: true });
		}
	}

	async function startTelegramQrLoginFromWizard() {
		if (isTelegramActionSubmitting) return;
		releaseTelegramQrSession({ resetAutoStartKey: false });
		const sessionGeneration = telegramQrSessionGeneration;
		isTelegramActionSubmitting = true;
		setupMessage = '';
		setupError = '';
		telegramError = '';
		try {
			await waitForTelegramQrSessionRelease();
			if (sessionGeneration !== telegramQrSessionGeneration || !isTelegramQrStepVisible()) return;
			const capabilities = await ensureTelegramCapabilities();
			if (sessionGeneration !== telegramQrSessionGeneration || !isTelegramQrStepVisible()) return;
			if (!capabilities) throw new Error('Telegram backend capabilities could not be loaded');
			const result = await startTelegramQrLoginFromWizardService({
				accountForm: telegramAccountForm,
				capabilities,
				externalAccountId: telegramWizardExternalAccountId()
			});
			if (sessionGeneration !== telegramQrSessionGeneration || !isTelegramQrStepVisible()) {
				if (result.qrLogin?.setup_id) void cancelTelegramQrBackendSession(result.qrLogin.setup_id);
				return;
			}
			if (result.error || !result.qrLogin) throw new Error(result.error || 'Telegram QR login start failed');
			applyTelegramQrLoginResult(result.qrLogin);
			if (result.qrLogin.status === 'ready') {
				await saveReadyTelegramQrAccountFromWizard();
			} else {
				scheduleTelegramQrStatusPolling(result.qrLogin);
				setupMessage =
					result.qrLogin.status === 'waiting_qr_scan' && result.qrLogin.qr_svg
						? 'Scan the Telegram QR code to continue'
						: result.qrLogin.message ?? `Telegram QR login status: ${result.qrLogin.status}`;
			}
		} catch (error) {
			if (sessionGeneration !== telegramQrSessionGeneration || !isTelegramQrStepVisible()) return;
			setupError = error instanceof Error ? error.message : 'Telegram QR login start failed';
			telegramError = setupError;
		} finally {
			isTelegramActionSubmitting = false;
		}
	}

	async function submitTelegramQrPasswordFromWizard() {
		if (isTelegramActionSubmitting || !telegramQrLogin) return;
		const setupId = telegramQrLogin.setup_id;
		const sessionGeneration = telegramQrSessionGeneration;
		isTelegramActionSubmitting = true;
		setupError = '';
		telegramError = '';
		try {
			const result = await submitTelegramQrPasswordFromWizardService(telegramQrLogin, telegramQrPassword);
			if (sessionGeneration !== telegramQrSessionGeneration || telegramQrLogin?.setup_id !== setupId || !isTelegramQrStepVisible()) return;
			if (result.error || !result.qrLogin) throw new Error(result.error || 'Telegram QR login password submit failed');
			telegramQrPassword = '';
			applyTelegramQrLoginResult(result.qrLogin);
			if (result.qrLogin.status === 'ready') {
				await saveReadyTelegramQrAccountFromWizard();
			} else {
				scheduleTelegramQrStatusPolling(result.qrLogin);
				setupMessage = result.message;
			}
		} catch (error) {
			if (sessionGeneration !== telegramQrSessionGeneration || !isTelegramQrStepVisible()) return;
			setupError = error instanceof Error ? error.message : 'Telegram QR login password submit failed';
			telegramError = setupError;
		} finally {
			isTelegramActionSubmitting = false;
		}
	}

	function submitTelegramQrStepFromWizard() {
		if (nextTelegramQrStepFromWizard(telegramQrLogin) === 'password') {
			void submitTelegramQrPasswordFromWizard();
			return;
		}
		void startTelegramQrLoginFromWizard();
	}

	async function refreshTelegramQrLoginStatus(options: { silent?: boolean } = {}) {
		if (!telegramQrLogin || isTelegramActionSubmitting || telegramQrStatusRequestInFlight) return;
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
			const result = await refreshTelegramQrLoginStatusService(telegramQrLogin);
			if (sessionGeneration !== telegramQrSessionGeneration || telegramQrLogin?.setup_id !== setupId || !isTelegramQrStepVisible()) return;
			if (result.error || !result.qrLogin) throw new Error(result.error || 'Telegram QR login status request failed');
			if (options.silent && telegramQrLogin.status === 'waiting_qr_scan' && result.qrLogin.status === 'waiting_qr_scan' && telegramQrLogin.qr_svg && result.qrLogin.qr_svg) {
				scheduleTelegramQrStatusPolling(result.qrLogin);
				return;
			}
			applyTelegramQrLoginResult(result.qrLogin);
			if (result.qrLogin.status === 'ready') {
				await saveReadyTelegramQrAccountFromWizard();
			} else {
				scheduleTelegramQrStatusPolling(result.qrLogin);
				const didReceiveFirstQrSvg = result.qrLogin.status === 'waiting_qr_scan' && !hadQrSvg && Boolean(result.qrLogin.qr_svg);
				if (!options.silent || result.qrLogin.status !== 'waiting_qr_scan' || didReceiveFirstQrSvg) {
					setupMessage = didReceiveFirstQrSvg ? 'Scan the Telegram QR code to continue' : result.message;
				}
			}
		} catch (error) {
			if (sessionGeneration !== telegramQrSessionGeneration || !isTelegramQrStepVisible()) return;
			setupError = error instanceof Error ? error.message : 'Telegram QR login status request failed';
			telegramError = setupError;
			clearTelegramQrStatusPolling();
		} finally {
			telegramQrStatusRequestInFlight = false;
			if (!options.silent) {
				isTelegramActionSubmitting = false;
			}
		}
	}
</script>

<TelegramAccountWizard
	bind:telegramAccountForm
	bind:telegramQrPassword
	{telegramWizardStep}
	{telegramAuthMethod}
	{telegramQrLogin}
	{telegramQrIsPreparing}
	{telegramNeedsFormAppCredentials}
	{isTelegramActionSubmitting}
	telegramWizardNote={telegramWizardNote()}
	{selectTelegramProviderKind}
	{selectTelegramAuthMethod}
	{continueTelegramWizard}
	{submitTelegramQrStepFromWizard}
	saveTelegramAccountFromWizard={() => saveTelegramAccountFromWizard()}
	{closeAccountDrawer}
	{telegramQrStatusLabel}
/>
{#if setupMessage}<p class="setup-state success">{setupMessage}</p>{/if}
{#if setupError}<p class="setup-state error">{setupError}</p>{/if}
