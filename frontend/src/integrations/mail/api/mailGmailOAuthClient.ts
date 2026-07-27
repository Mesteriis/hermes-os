import { create } from '@bufbuild/protobuf'
import { createClient, type Client } from '@connectrpc/connect'

import {
	CompleteGmailOAuthRequestV1Schema,
	GetGmailOAuthStatusRequestV1Schema,
	type GmailOAuthOperationStatusV1,
	type GmailOAuthStartedV1,
	GmailOAuthCompleteService,
	GmailOAuthQueryService,
	GmailOAuthStartService,
	type MailAcceptedV1,
	StartGmailOAuthRequestV1Schema,
} from '../../../gen/hermes/mail/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

export class MailGmailOAuthClientV1 {
	constructor(
		private readonly startClient: Client<typeof GmailOAuthStartService> = createClient(
			GmailOAuthStartService,
			createBrowserGatewayConnectTransport(),
		),
		private readonly completeClient: Client<typeof GmailOAuthCompleteService> = createClient(
			GmailOAuthCompleteService,
			createBrowserGatewayConnectTransport(),
		),
		private readonly queryClient: Client<typeof GmailOAuthQueryService> = createClient(
			GmailOAuthQueryService,
			createBrowserGatewayConnectTransport(),
		),
	) {}

	async start(operationId: string): Promise<GmailOAuthStartedV1> {
		validateOperationId(operationId)
		return this.startClient.start(create(
			StartGmailOAuthRequestV1Schema,
			{ operationId },
		))
	}

	async complete(input: {
		operationId: string
		setupId: string
		state: string
		authorizationCode: string
	}): Promise<MailAcceptedV1> {
		validateOperationId(input.operationId)
		for (const value of [input.setupId, input.state, input.authorizationCode]) {
			if (value.trim().length === 0) throw new Error('Gmail OAuth completion input is invalid')
		}
		return this.completeClient.complete(create(
			CompleteGmailOAuthRequestV1Schema,
			input,
		))
	}

	async status(operationId: string): Promise<GmailOAuthOperationStatusV1 | undefined> {
		validateOperationId(operationId)
		return (await this.queryClient.getOperationStatus(create(
			GetGmailOAuthStatusRequestV1Schema,
			{ operationId },
		))).status
	}
}

function validateOperationId(value: string): void {
	if (value.trim().length === 0 || value.length > 128) {
		throw new Error('Gmail OAuth operation id is invalid')
	}
}
