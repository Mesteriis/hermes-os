import type { ProviderAccountNavigationSnapshot } from '../../../shared/ui/shell/providerAccountNavigation'
import type { MailAccountConnection } from '../queries/mailAccountConnections'

export function mailAccountNavigation(
	connections: readonly MailAccountConnection[],
	selectedConnectionId: string,
	loading: boolean,
): ProviderAccountNavigationSnapshot {
	return {
		channelId: 'mail',
		entries: connections.map((connection) => ({
			accountId: connection.connectionId,
			label: connection.connectionId,
		})),
		loading,
		selectedAccountId: selectedConnectionId,
	}
}
