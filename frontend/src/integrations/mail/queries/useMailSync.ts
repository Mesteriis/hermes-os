import { computed, ref } from 'vue'
import { syncMailInbox } from '../api/mailOperationalGateway'
import type { MailSyncModel } from '../presentation/mailSyncModel'

export function useMailSync(capabilities: {
	canSync: () => boolean
	connectionId: () => string
}) {
	const busy = ref(false)
	const notice = ref('')
	const summary = ref('')

	const model = computed<MailSyncModel>(() => ({
		busy: busy.value,
		canSync: capabilities.canSync() && Boolean(capabilities.connectionId()),
		notice: notice.value,
		summary: summary.value,
	}))

	async function sync(): Promise<void> {
		if (!capabilities.canSync()) {
			notice.value = 'Mail sync capability is not admitted.'
			return
		}
		const connectionId = capabilities.connectionId()
		if (!connectionId) {
			notice.value = 'Select a ready Mail account before syncing.'
			return
		}
		busy.value = true
		notice.value = ''
		try {
			const result = await syncMailInbox(connectionId, crypto.randomUUID())
			summary.value = `${result.observedMessages} messages observed by ${result.operationId}.`
		} catch (error) {
			notice.value = error instanceof Error ? error.message : 'Mail sync failed.'
		} finally {
			busy.value = false
		}
	}

	return { model, sync }
}
