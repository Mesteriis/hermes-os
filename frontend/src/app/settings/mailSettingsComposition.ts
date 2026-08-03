import type { MailAccountConnection } from '../../integrations/mail/queries/mailAccountConnections'
import type { MailContactsSyncAccountChoiceV1 } from '../../workflows/mail-contacts-sync/queries/useMailContactsSyncSettings'

export function mailContactsSyncAccountChoices(
	connections: readonly MailAccountConnection[],
): readonly MailContactsSyncAccountChoiceV1[] {
	return connections.map((connection) => ({
		accountId: connection.connectionId,
		syncReady: connection.syncReady,
	}))
}
