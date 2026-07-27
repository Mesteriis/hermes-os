import { create } from '@bufbuild/protobuf'
import { createClient, type Client } from '@connectrpc/connect'

import {
	ZulipAccountLifecycleCommandV1Schema,
	type ZulipAccountLifecycleReceiptV1,
	ZulipAccountLifecycleService,
	ZulipBindCredentialV1Schema,
} from '../../../gen/hermes/zulip/account/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let lifecycleClient: Client<typeof ZulipAccountLifecycleService> | null = null

export async function bindZulipCredential(input: {
	accountId: string
	expectedBindingRevision: bigint
	credentialRevision: bigint
}): Promise<ZulipAccountLifecycleReceiptV1> {
	return getZulipAccountLifecycleConnectClient().apply(create(
		ZulipAccountLifecycleCommandV1Schema,
		{
			command: {
				case: 'bindCredential',
				value: create(ZulipBindCredentialV1Schema, input),
			},
		},
	))
}

export function getZulipAccountLifecycleConnectClient(): Client<
	typeof ZulipAccountLifecycleService
> {
	if (!lifecycleClient) {
		lifecycleClient = createClient(
			ZulipAccountLifecycleService,
			createBrowserGatewayConnectTransport(),
		)
	}
	return lifecycleClient
}

export function resetZulipAccountLifecycleConnectClientForTests(): void {
	lifecycleClient = null
}
