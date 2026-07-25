import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { MailDeliveryService } from '../../../gen/hermes/mail/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let mailDeliveryClient: Client<typeof MailDeliveryService> | null = null

function createMailDeliveryConnectClient(): Client<typeof MailDeliveryService> {
	return createClient(MailDeliveryService, createBrowserGatewayConnectTransport())
}

export function getMailDeliveryConnectClient(): Client<typeof MailDeliveryService> {
	if (!mailDeliveryClient) {
		mailDeliveryClient = createMailDeliveryConnectClient()
	}

	return mailDeliveryClient
}

export function resetMailDeliveryConnectClientForTests(): void {
	mailDeliveryClient = null
}
