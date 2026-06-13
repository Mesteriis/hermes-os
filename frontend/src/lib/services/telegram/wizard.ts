import {
	fetchTelegramQrLoginStatus,
	setupTelegramAccount,
	setupTelegramFixtureAccount,
	startTelegramQrLogin,
	submitTelegramQrLoginPassword,
	type TelegramAccountSetupResponse,
	type TelegramCapabilitiesResponse,
	type TelegramLiveAccountSetupRequest,
	type TelegramProviderKind,
	type TelegramQrLoginStatusResponse
} from '$lib/api';

export async function saveTelegramAccountFromWizard(params: {
	accountForm: {
		account_id: string;
		provider_kind: TelegramProviderKind;
		display_name: string;
		external_account_id: string;
		api_id: string;
		api_hash: string;
		bot_token: string;
		session_encryption_key: string;
		tdlib_data_path: string;
		transcription_enabled: boolean;
	};
	authMethod: string;
	qrLogin: TelegramQrLoginStatusResponse | null;
	isFixtureSetup: boolean;
}): Promise<{
	message: string;
	error: string;
	accountId: string;
	providerKind: string;
}> {
	const { accountForm, authMethod, qrLogin, isFixtureSetup } = params;
	const providerKind =
		authMethod === 'bot_token'
			? 'telegram_bot'
			: authMethod === 'phone' || authMethod === 'qr'
				? 'telegram_user'
				: accountForm.provider_kind;
	const isQrAuthorizedAccount = authMethod === 'qr' && qrLogin?.status === 'ready';

	try {
		let result: TelegramAccountSetupResponse;
		if (isFixtureSetup) {
			result = await setupTelegramFixtureAccount({
				account_id: accountForm.account_id,
				provider_kind: providerKind,
				display_name: accountForm.display_name,
				external_account_id: accountForm.external_account_id,
				tdlib_data_path: accountForm.tdlib_data_path || undefined,
				transcription_enabled: authMethod === 'qr' ? false : accountForm.transcription_enabled
			});
		} else {
			const request: TelegramLiveAccountSetupRequest = {
				account_id: accountForm.account_id,
				provider_kind: providerKind,
				display_name: accountForm.display_name,
				external_account_id: accountForm.external_account_id,
				tdlib_data_path: accountForm.tdlib_data_path || undefined,
				transcription_enabled: authMethod === 'qr' ? false : accountForm.transcription_enabled
			};
			if (providerKind === 'telegram_user' && isQrAuthorizedAccount) {
				request.qr_authorized = true;
				if (accountForm.session_encryption_key) {
					request.session_encryption_key = accountForm.session_encryption_key;
				}
			} else if (providerKind === 'telegram_user') {
				const apiId = accountForm.api_id.trim();
				const apiHash = accountForm.api_hash.trim();
				if (apiId) {
					request.api_id = Number(apiId);
				}
				if (apiHash) {
					request.api_hash = apiHash;
				}
				if (accountForm.session_encryption_key) {
					request.session_encryption_key = accountForm.session_encryption_key;
				}
			} else if (providerKind === 'telegram_bot' && accountForm.bot_token) {
				request.bot_token = accountForm.bot_token;
			}
			result = await setupTelegramAccount(request);
		}
		const runtimeLabel =
			authMethod === 'qr' && qrLogin?.status === 'ready'
				? 'saved after QR authorization'
				: result.runtime === 'live_blocked'
					? 'saved as live-blocked'
					: 'saved';
		const message = `${providerKindLabel(result.provider_kind)} account ${result.account_id} ${runtimeLabel}`;
		return {
			message,
			error: '',
			accountId: result.account_id,
			providerKind: result.provider_kind
		};
	} catch (error) {
		return {
			message: '',
			error: error instanceof Error ? error.message : 'Telegram account setup failed',
			accountId: accountForm.account_id,
			providerKind
		};
	}
}

export async function saveReadyTelegramQrAccountFromWizard(
	authMethod: string,
	qrLogin: TelegramQrLoginStatusResponse | null
): Promise<{ shouldSave: boolean }> {
	if (authMethod !== 'qr' || qrLogin?.status !== 'ready') {
		return { shouldSave: false };
	}
	return { shouldSave: true };
}

export async function startTelegramQrLoginFromWizard(params: {
	accountForm: {
		account_id: string;
		display_name: string;
		api_id: string;
		api_hash: string;
		session_encryption_key: string;
		tdlib_data_path?: string;
	};
	capabilities: TelegramCapabilitiesResponse | null;
	externalAccountId: string;
}): Promise<{
	qrLogin: TelegramQrLoginStatusResponse | null;
	message: string;
	error: string;
}> {
	const { accountForm, capabilities, externalAccountId } = params;

	if (capabilities && !capabilities.tdjson_runtime_available) {
		return {
			qrLogin: null,
			message: '',
			error: 'TDLib JSON runtime is not available in the running backend'
		};
	}

	if (capabilities && !capabilities.telegram_app_credentials_configured) {
		return {
			qrLogin: null,
			message: '',
			error: 'Telegram app credentials are not configured in the backend environment'
		};
	}

	try {
		const result = await startTelegramQrLogin({
			account_id: accountForm.account_id,
			display_name: accountForm.display_name,
			external_account_id: externalAccountId,
			session_encryption_key: accountForm.session_encryption_key || undefined,
			tdlib_data_path: accountForm.tdlib_data_path || undefined,
			transcription_enabled: false
		});
		const message =
			result.status === 'waiting_qr_scan'
				? 'Scan the Telegram QR code to continue'
				: result.message ?? `Telegram QR login status: ${result.status}`;
		return { qrLogin: result, message, error: '' };
	} catch (error) {
		return {
			qrLogin: null,
			message: '',
			error: error instanceof Error ? error.message : 'Telegram QR login start failed'
		};
	}
}

export async function submitTelegramQrPasswordFromWizard(
	qrLogin: TelegramQrLoginStatusResponse,
	password: string
): Promise<{
	qrLogin: TelegramQrLoginStatusResponse | null;
	message: string;
	error: string;
}> {
	if (qrLogin.status !== 'waiting_password') {
		return {
			qrLogin: null,
			message: '',
			error: 'Telegram QR login is not waiting for a password'
		};
	}

	if (!password) {
		return {
			qrLogin: null,
			message: '',
			error: 'Telegram 2-step verification password is required'
		};
	}

	try {
		const result = await submitTelegramQrLoginPassword(
			qrLogin.setup_id,
			{ password }
		);
		const message = result.message ?? `Telegram QR login status: ${result.status}`;
		return { qrLogin: result, message, error: '' };
	} catch (error) {
		return {
			qrLogin: null,
			message: '',
			error: error instanceof Error ? error.message : 'Telegram QR login password submit failed'
		};
	}
}

export function shouldPollTelegramQrLoginStatus(
	status: TelegramQrLoginStatusResponse['status'] | null | undefined
) {
	return status === 'waiting_qr_scan' || status === 'waiting_password';
}

export function submitTelegramQrStepFromWizard(qrLogin: TelegramQrLoginStatusResponse | null) {
	if (qrLogin?.status === 'waiting_password') {
		return 'password';
	}
	return 'start';
}

export async function refreshTelegramQrLoginStatus(
	qrLogin: TelegramQrLoginStatusResponse
): Promise<{
	qrLogin: TelegramQrLoginStatusResponse | null;
	message: string;
	error: string;
}> {
	try {
		const result = await fetchTelegramQrLoginStatus(
			qrLogin.setup_id
		);
		const message = result.message ?? `Telegram QR login status: ${result.status}`;
		return { qrLogin: result, message, error: '' };
	} catch (error) {
		return {
			qrLogin: null,
			message: '',
			error: error instanceof Error ? error.message : 'Telegram QR login status request failed'
		};
	}
}

function providerKindLabel(value: string) {
	return value
		.split('_')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}
