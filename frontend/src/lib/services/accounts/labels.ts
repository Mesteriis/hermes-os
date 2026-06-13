import type { ProviderAccount } from '$lib/api';
import { formatDateTime } from '../formatting';

export function accountProviderIcon(providerKind: string) {
	if (providerKind === 'telegram_user' || providerKind === 'telegram_bot') {
		return 'tabler:brand-telegram';
	}
	if (providerKind === 'whatsapp_web') {
		return 'tabler:brand-whatsapp';
	}
	return 'tabler:mail';
}

export function accountProviderLabel(providerKind: string) {
	return providerKind
		.split('_')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}

export function accountUpdatedLabel(account: ProviderAccount) {
	return formatDateTime(account.updated_at) || 'Never';
}

export function providerKindLabel(value: string) {
	return value
		.split('_')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}

export function capabilityLabel(value: string) {
	return value
		.split('_')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}
