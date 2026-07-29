import type { ProviderAccountNavigationSnapshot } from '../../../shared/ui/shell/providerAccountNavigation'
import type { TelegramAccountAccessModel } from './telegramAccountAccessModel'

export function telegramAccountNavigation(
	model: TelegramAccountAccessModel,
): ProviderAccountNavigationSnapshot {
	return {
		channelId: 'telegram',
		entries: model.accounts.map((account) => ({
			accountId: account.id,
			label: account.title,
		})),
		loading: model.pending,
		selectedAccountId: model.selectedAccountId,
	}
}
