import { computed, ref } from 'vue'
import { defineStore } from 'pinia'

export type RealtimeTransportKind = 'websocket' | 'sse' | 'long_poll'
export type RealtimeTransportState =
	| 'idle'
	| 'connecting'
	| 'connected'
	| 'reconnecting'
	| 'fallback'
	| 'disconnected'
export type RealtimeStatusTone = 'neutral' | 'success' | 'warning' | 'danger'

export type RealtimeStatusUpdate = {
	transport: RealtimeTransportKind
	state: Exclude<RealtimeTransportState, 'idle'>
	attempt?: number
	maxAttempts?: number
	error?: string
}

export type RealtimeStatusSnapshot = {
	transport: RealtimeTransportKind
	state: RealtimeTransportState
	attempt: number | null
	maxAttempts: number | null
	error: string | null
	updatedAt: string | null
}

const initialStatus: RealtimeStatusSnapshot = {
	transport: 'websocket',
	state: 'idle',
	attempt: null,
	maxAttempts: null,
	error: null,
	updatedAt: null
}

export const useRealtimeStatusStore = defineStore('realtimeStatus', () => {
	const status = ref<RealtimeStatusSnapshot>({ ...initialStatus })

	const isRealtimeDegraded = computed<boolean>(() => {
		return status.value.state === 'reconnecting' || status.value.transport === 'long_poll'
	})

	const realtimeStatusLabel = computed<string>(() => {
		if (status.value.state === 'idle') return 'Realtime starting'
		if (status.value.state === 'connecting') return 'Realtime connecting'
		if (status.value.state === 'connected') {
			return status.value.transport === 'long_poll' ? 'Realtime fallback' : 'Realtime live'
		}
		if (status.value.state === 'reconnecting') return 'Realtime reconnecting'
		if (status.value.state === 'fallback') return 'Realtime fallback'
		return 'Realtime offline'
	})

	const realtimeStatusTone = computed<RealtimeStatusTone>(() => {
		if (status.value.state === 'connected' && status.value.transport !== 'long_poll') {
			return 'success'
		}
		if (
			status.value.state === 'reconnecting' ||
			status.value.state === 'fallback' ||
			status.value.transport === 'long_poll'
		) {
			return 'warning'
		}
		if (status.value.state === 'disconnected') return 'danger'
		return 'neutral'
	})

	const realtimeStatusDetail = computed<string>(() => {
		const base = realtimeStatusLabel.value
		if (status.value.error) return `${base}: ${status.value.error}`
		if (status.value.attempt && status.value.maxAttempts) {
			return `${base}: attempt ${status.value.attempt}/${status.value.maxAttempts}`
		}
		return base
	})

	function setRealtimeStatus(update: RealtimeStatusUpdate): void {
		status.value = {
			transport: update.transport,
			state: update.state,
			attempt: update.attempt ?? null,
			maxAttempts: update.maxAttempts ?? null,
			error: update.error?.trim() || null,
			updatedAt: new Date().toISOString()
		}
	}

	function resetRealtimeStatus(): void {
		status.value = { ...initialStatus }
	}

	return {
		status,
		isRealtimeDegraded,
		realtimeStatusLabel,
		realtimeStatusTone,
		realtimeStatusDetail,
		setRealtimeStatus,
		resetRealtimeStatus
	}
})
