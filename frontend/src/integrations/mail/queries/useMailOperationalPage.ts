import { computed, ref } from 'vue'
import type { MailDeliveryOperationStatusV1 } from '../../../gen/hermes/mail/v1/client_pb'
import {
	getMailDeliveryStatus,
	sendMailMessage,
	syncMailInbox,
} from '../api/mailOperationalGateway'
import {
	buildMailDeliveryStatusCard,
	type MailOperationalPageModel,
} from '../presentation/mailOperationalPageModel'

export function useMailOperationalPage(capabilities: {
	canDeliver: () => boolean
	canSync: () => boolean
}) {
	const recipients = ref('')
	const subject = ref('')
	const textBody = ref('')
	const providerConversationId = ref('')
	const operationId = ref('')
	const busyAction = ref<MailOperationalPageModel['busyAction']>(null)
	const notice = ref('')
	const syncSummary = ref('')
	const status = ref<MailDeliveryOperationStatusV1 | null>(null)

	const model = computed<MailOperationalPageModel>(() => ({
		recipients: recipients.value,
		subject: subject.value,
		textBody: textBody.value,
		providerConversationId: providerConversationId.value,
		operationId: operationId.value,
		busyAction: busyAction.value,
		canDeliver: capabilities.canDeliver(),
		canSync: capabilities.canSync(),
		notice: notice.value,
		syncSummary: syncSummary.value,
		status: buildMailDeliveryStatusCard(status.value),
	}))

	async function sync(): Promise<void> {
		if (!capabilities.canSync()) {
			notice.value = 'Mail sync capability is not admitted.'
			return
		}
		await run('sync', async () => {
			const result = await syncMailInbox(crypto.randomUUID())
			syncSummary.value = `${result.observedMessages} messages observed by ${result.operationId}.`
		})
	}

	async function deliver(): Promise<void> {
		if (!capabilities.canDeliver()) {
			notice.value = 'Mail delivery capability is not admitted.'
			return
		}
		await run('delivery', async () => {
			operationId.value = await sendMailMessage({
				operationId: crypto.randomUUID(),
				providerConversationId: providerConversationId.value,
				recipients: recipients.value.split(/[\n,;]/),
				subject: subject.value,
				textBody: textBody.value,
			})
			textBody.value = ''
			notice.value = `Mail operation ${operationId.value} accepted.`
		})
	}

	async function refreshStatus(): Promise<void> {
		await run('status', async () => {
			status.value = await getMailDeliveryStatus(operationId.value)
			if (!status.value) notice.value = 'No Mail delivery was found for this operation ID.'
		})
	}

	async function run(
		action: NonNullable<MailOperationalPageModel['busyAction']>,
		work: () => Promise<void>,
	): Promise<void> {
		busyAction.value = action
		notice.value = ''
		try {
			await work()
		} catch (error) {
			notice.value = error instanceof Error ? error.message : 'Mail operation failed.'
		} finally {
			busyAction.value = null
		}
	}

	return {
		model,
		sync,
		deliver,
		refreshStatus,
		updateRecipients: (value: string) => { recipients.value = value },
		updateSubject: (value: string) => { subject.value = value },
		updateTextBody: (value: string) => { textBody.value = value },
		updateProviderConversationId: (value: string) => { providerConversationId.value = value },
		updateOperationId: (value: string) => { operationId.value = value },
	}
}
