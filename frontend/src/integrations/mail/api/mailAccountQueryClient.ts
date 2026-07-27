import { create } from '@bufbuild/protobuf'
import { createClient, type Client } from '@connectrpc/connect'

import {
	MailAccountQueryService,
	MailAccountStatusRequestV1Schema,
	type MailAccountStatusV1,
} from '../../../gen/hermes/mail/account/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let client: Client<typeof MailAccountQueryService> | null = null

export async function getMailAccountStatus(
	connectionId: string,
): Promise<MailAccountStatusV1> {
	if (connectionId.trim().length === 0) throw new Error('mail connection id is invalid')
	return getMailAccountQueryConnectClient().get(create(
		MailAccountStatusRequestV1Schema,
		{ connectionId },
	))
}

export function getMailAccountQueryConnectClient(): Client<typeof MailAccountQueryService> {
	client ??= createClient(
		MailAccountQueryService,
		createBrowserGatewayConnectTransport(),
	)
	return client
}

export function resetMailAccountQueryConnectClientForTests(): void {
	client = null
}
