import { createClient, type Client } from '@connectrpc/connect'

import { MailCompositionCommandService } from '../../../gen/hermes/mail/composition/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let mailCompositionCommandClient: Client<typeof MailCompositionCommandService> | null = null

function createMailCompositionCommandConnectClient(): Client<typeof MailCompositionCommandService> {
	return createClient(
		MailCompositionCommandService,
		createBrowserGatewayConnectTransport({ defaultTimeoutMs: 15_000 }),
	)
}

export function getMailCompositionCommandConnectClient(): Client<typeof MailCompositionCommandService> {
	if (!mailCompositionCommandClient) {
		mailCompositionCommandClient = createMailCompositionCommandConnectClient()
	}
	return mailCompositionCommandClient
}

export function resetMailCompositionCommandConnectClientForTests(): void {
	mailCompositionCommandClient = null
}
