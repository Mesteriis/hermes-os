import {
	logoutTelegramAccount,
	removeTelegramAccount,
	type TelegramAccountLifecycleResponse
} from '$lib/api';

export async function logoutTelegramAccountFromUi(accountId: string): Promise<{
	result: TelegramAccountLifecycleResponse | null;
	message: string;
	error: string;
}> {
	try {
		const result = await logoutTelegramAccount(accountId);
		return {
			result,
			message: `Telegram account ${result.account.account_id} logged out`,
			error: ''
		};
	} catch (error) {
		return {
			result: null,
			message: '',
			error: error instanceof Error ? error.message : 'Telegram account logout failed'
		};
	}
}

export async function removeTelegramAccountFromUi(accountId: string): Promise<{
	result: TelegramAccountLifecycleResponse | null;
	message: string;
	error: string;
}> {
	try {
		const result = await removeTelegramAccount(accountId);
		return {
			result,
			message: `Telegram account ${result.account.account_id} removed from active accounts`,
			error: ''
		};
	} catch (error) {
		return {
			result: null,
			message: '',
			error: error instanceof Error ? error.message : 'Telegram account remove failed'
		};
	}
}
