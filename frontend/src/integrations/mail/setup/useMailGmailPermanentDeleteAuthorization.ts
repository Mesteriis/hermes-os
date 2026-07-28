import { computed, ref } from 'vue'
import {
	GmailOAuthOutcomeV1,
	type GmailOAuthStartedV1,
} from '../../../gen/hermes/mail/v1/client_pb'
import { MailGmailOAuthClientV1 } from '../api/mailGmailOAuthClient'

export function useMailGmailPermanentDeleteAuthorization() {
	const client = new MailGmailOAuthClientV1()
	const connectionId = ref('')
	const operationId = ref('')
	const started = ref<GmailOAuthStartedV1>()
	const returnedState = ref('')
	const authorizationCode = ref('')
	const busy = ref(false)
	const message = ref('')
	const failed = ref(false)
	const submitLabel = computed(() =>
		started.value ? 'Complete authorization' : 'Authorize deletion',
	)

	async function submit(): Promise<void> {
		busy.value = true
		failed.value = false
		message.value = ''
		try {
			if (!started.value) {
				operationId.value = `mail-gmail-permanent-delete-auth-${crypto.randomUUID()}`
				started.value = await client.start(
					operationId.value,
					requiredConnectionId(connectionId.value),
					'permanent-delete',
				)
				message.value = 'Open Google authorization, then paste the returned state and code.'
				return
			}
			await client.complete({
				operationId: operationId.value,
				connectionId: requiredConnectionId(connectionId.value),
				setupId: started.value.setupId,
				state: returnedState.value,
				authorizationCode: authorizationCode.value,
			})
			authorizationCode.value = ''
			message.value = 'Authorization accepted; refresh status after provider exchange.'
		} catch (error) {
			failed.value = true
			message.value = error instanceof Error ? error.message : 'Gmail authorization failed.'
		} finally {
			busy.value = false
		}
	}

	async function refreshStatus(): Promise<void> {
		if (!operationId.value) return
		busy.value = true
		failed.value = false
		try {
			const status = await client.status(
				operationId.value,
				requiredConnectionId(connectionId.value),
			)
			if (status?.outcome === GmailOAuthOutcomeV1.GMAIL_OAUTH_OUTCOME_COMPLETED) {
				message.value = 'Permanent-delete authority is active for this Gmail account.'
				return
			}
			if (status?.outcome === GmailOAuthOutcomeV1.GMAIL_OAUTH_OUTCOME_REJECTED) {
				failed.value = true
				message.value = 'Google did not grant the required permanent-delete authority.'
				return
			}
			message.value = 'Gmail authorization is still pending.'
		} catch (error) {
			failed.value = true
			message.value = error instanceof Error
				? error.message
				: 'Gmail authorization status failed.'
		} finally {
			busy.value = false
		}
	}

	return {
		authorizationCode,
		busy,
		connectionId,
		failed,
		message,
		operationId,
		refreshStatus,
		returnedState,
		started,
		submit,
		submitLabel,
	}
}

function requiredConnectionId(value: string): string {
	const connectionId = value.trim()
	if (!connectionId) throw new Error('Select a Gmail account before authorization.')
	return connectionId
}
