import { computed, ref } from 'vue'

import { retryTelegramOperation } from '../api/telegramLifecycleGateway'
import type { TelegramOperationRetryModel } from '../presentation/telegramOperationRetryModel'

export function useTelegramOperationRetry(canRetry: () => boolean) {
	const operationId = ref('')
	const pending = ref(false)
	const statusMessage = ref('')
	const model = computed<TelegramOperationRetryModel>(() => ({
		operationId: operationId.value,
		pending: pending.value,
		statusMessage: statusMessage.value,
		canRetry: canRetry(),
	}))

	async function retry(): Promise<void> {
		if (!canRetry()) {
			statusMessage.value = 'Telegram lifecycle capability is not admitted.'
			return
		}
		pending.value = true
		statusMessage.value = ''
		try {
			const operation = await retryTelegramOperation(
				operationId.value,
				BigInt(Math.floor(Date.now() / 1_000)),
			)
			statusMessage.value = `Retry ${operation.operationId} is ${operation.state || 'accepted'}.`
		} catch (error) {
			statusMessage.value = error instanceof Error ? error.message : 'Telegram retry failed.'
		} finally {
			pending.value = false
		}
	}

	function updateOperationId(value: string): void {
		operationId.value = value
	}

	return { model, retry, updateOperationId }
}
