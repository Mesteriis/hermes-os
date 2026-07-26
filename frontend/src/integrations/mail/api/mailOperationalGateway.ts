import type {
	MailDeliveryOperationStatusV1,
	SyncInboxCompletedV1,
} from '../../../gen/hermes/mail/v1/client_pb'
import { getMailDeliveryCommandConnectClient } from './mailDeliveryCommandClient'
import { getMailDeliveryQueryConnectClient } from './mailDeliveryQueryClient'
import { getMailSyncConnectClient } from './mailSyncClient'

export async function syncMailInbox(operationId: string): Promise<SyncInboxCompletedV1> {
	return getMailSyncConnectClient().sync({
		operationId: requireIdentifier('operation ID', operationId),
	})
}

export async function sendMailMessage(input: {
	operationId: string
	providerConversationId: string
	recipients: readonly string[]
	subject: string
	textBody: string
}): Promise<string> {
	const recipients = input.recipients.map((recipient) => recipient.trim()).filter(Boolean)
	if (recipients.length === 0) {
		throw new RangeError('Mail recipient is required')
	}
	const textBody = input.textBody.trim()
	if (!textBody) {
		throw new RangeError('Mail body is required')
	}
	const response = await getMailDeliveryCommandConnectClient().send({
		operationId: requireIdentifier('operation ID', input.operationId),
		providerConversationId: input.providerConversationId.trim(),
		recipient: recipients,
		subject: input.subject.trim(),
		textBody,
		attachmentAnchorId: [],
	})
	return response.operationId
}

export async function getMailDeliveryStatus(
	operationId: string,
): Promise<MailDeliveryOperationStatusV1 | null> {
	const response = await getMailDeliveryQueryConnectClient().getOperationStatus({
		operationId: requireIdentifier('operation ID', operationId),
	})
	return response.status ?? null
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError(`Mail ${label} is required`)
	}
	return normalized
}
