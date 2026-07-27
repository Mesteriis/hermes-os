import { computed, ref } from 'vue'
import { syncMailInbox } from '../api/mailOperationalGateway'
import type { MailSyncModel } from '../presentation/mailSyncModel'

export function useMailSync(capabilities: { canSync: () => boolean }) {
	const busy = ref(false)
	const notice = ref('')
	const summary = ref('')

	const model = computed<MailSyncModel>(() => ({
		busy: busy.value,
		canSync: capabilities.canSync(),
		notice: notice.value,
		summary: summary.value,
	}))

	async function sync(): Promise<void> {
		if (!capabilities.canSync()) {
			notice.value = 'Mail sync capability is not admitted.'
			return
		}
		busy.value = true
		notice.value = ''
		try {
			const result = await syncMailInbox(crypto.randomUUID())
			summary.value = `${result.observedMessages} messages observed by ${result.operationId}.`
		} catch (error) {
			notice.value = error instanceof Error ? error.message : 'Mail sync failed.'
		} finally {
			busy.value = false
		}
	}

	return { model, sync }
}
