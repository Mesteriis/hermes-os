import { createClient, type Client } from '@connectrpc/connect'

import { MailMessageFlagQueryService } from '../../../gen/hermes/mail/message_flags/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let mailMessageFlagQueryClient: Client<typeof MailMessageFlagQueryService> | null = null

function createMailMessageFlagQueryConnectClient(): Client<typeof MailMessageFlagQueryService> {
	return createClient(
		MailMessageFlagQueryService,
		createBrowserGatewayConnectTransport({ defaultTimeoutMs: 15_000 }),
	)
}

export function getMailMessageFlagQueryConnectClient(): Client<typeof MailMessageFlagQueryService> {
	if (!mailMessageFlagQueryClient) {
		mailMessageFlagQueryClient = createMailMessageFlagQueryConnectClient()
	}
	return mailMessageFlagQueryClient
}

export function resetMailMessageFlagQueryConnectClientForTests(): void {
	mailMessageFlagQueryClient = null
}
