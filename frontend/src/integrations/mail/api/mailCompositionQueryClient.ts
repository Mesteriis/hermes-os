import { createClient, type Client } from '@connectrpc/connect'

import { MailCompositionQueryService } from '../../../gen/hermes/mail/composition/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let mailCompositionQueryClient: Client<typeof MailCompositionQueryService> | null = null

function createMailCompositionQueryConnectClient(): Client<typeof MailCompositionQueryService> {
	return createClient(
		MailCompositionQueryService,
		createBrowserGatewayConnectTransport({ defaultTimeoutMs: 15_000 }),
	)
}

export function getMailCompositionQueryConnectClient(): Client<typeof MailCompositionQueryService> {
	if (!mailCompositionQueryClient) {
		mailCompositionQueryClient = createMailCompositionQueryConnectClient()
	}
	return mailCompositionQueryClient
}

export function resetMailCompositionQueryConnectClientForTests(): void {
	mailCompositionQueryClient = null
}
