import { computed, ref } from 'vue'
import { defineStore } from 'pinia'

export type RealtimeTransportKind = 'sse'
export type RealtimeTransportState =
	| 'idle'
	| 'connecting'
	| 'connected'
	| 'reconnecting'
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
	lastEventId: string | null
	lastEventAt: string | null
	lastLaggedSkipped: number | null
	lastLaggedAt: string | null
	updatedAt: string | null
}

const initialStatus: RealtimeStatusSnapshot = {
	transport: 'sse',
	state: 'idle',
	attempt: null,
	maxAttempts: null,
	error: null,
	lastEventId: null,
	lastEventAt: null,
	lastLaggedSkipped: null,
	lastLaggedAt: null,
	updatedAt: null
}

export const useRealtimeStatusStore = defineStore('realtimeStatus', () => {
	const status = ref<RealtimeStatusSnapshot>({ ...initialStatus })
	let reconnectHandler: (() => void) | null = null

	const isRealtimeDegraded = computed<boolean>(() => {
		return (
			status.value.state === 'reconnecting' || status.value.lastLaggedSkipped !== null
		)
	})

	const canTriggerReconnect = computed<boolean>(() => {
		return status.value.state === 'disconnected' || isRealtimeDegraded.value
	})

	const realtimeStatusLabel = computed<string>(() => {
		if (status.value.state === 'idle') return 'Realtime starting'
		if (status.value.state === 'connecting') return 'Realtime connecting'
		if (status.value.state === 'connected') {
			return 'Realtime live'
		}
		if (status.value.state === 'reconnecting') return 'Realtime reconnecting'
		return 'Realtime offline'
	})

	const realtimeStatusTone = computed<RealtimeStatusTone>(() => {
		if (status.value.state === 'connected') {
			return 'success'
		}
		if (
			status.value.state === 'reconnecting'
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

	const realtimeRecoveryDetail = computed<string>(() => {
		const cursor = status.value.lastEventId
		const laggedSkipped = status.value.lastLaggedSkipped
		if (laggedSkipped !== null) {
			const cursorLabel = cursor ?? 'unknown'
			return `Replay gap detected after cursor ${cursorLabel}. Skipped ${laggedSkipped} event${laggedSkipped === 1 ? '' : 's'}. Reconnect to recover missed updates.`
		}
		if (!cursor) {
			return 'Waiting for first replay cursor'
		}

		const lastEventAt = status.value.lastEventAt ? formatRecoveryTimestamp(status.value.lastEventAt) : 'unknown'
		if (status.value.state === 'disconnected') {
			return `Offline recovery will resume from cursor ${cursor}. Last event ${lastEventAt}`
		}
		if (isRealtimeDegraded.value) {
			return `Recovery cursor ${cursor}. Last event ${lastEventAt}`
		}
		return `Replay cursor ${cursor}. Last event ${lastEventAt}`
	})

	function setRealtimeStatus(update: RealtimeStatusUpdate): void {
		status.value = {
			transport: update.transport,
			state: update.state,
			attempt: update.attempt ?? null,
			maxAttempts: update.maxAttempts ?? null,
			error: update.error?.trim() || null,
			lastEventId: status.value.lastEventId,
			lastEventAt: status.value.lastEventAt,
			lastLaggedSkipped: update.state === 'connected' ? null : status.value.lastLaggedSkipped,
			lastLaggedAt: update.state === 'connected' ? null : status.value.lastLaggedAt,
			updatedAt: new Date().toISOString()
		}
	}

	function observeRealtimeEvent(eventId: string): void {
		const cursor = eventId.trim()
		if (!cursor) return

		status.value = {
			...status.value,
			lastEventId: cursor,
			lastEventAt: new Date().toISOString(),
			updatedAt: new Date().toISOString()
		}
	}

	function observeRealtimeLag(skipped: number): void {
		if (!Number.isFinite(skipped) || skipped <= 0) return

		status.value = {
			...status.value,
			lastLaggedSkipped: Math.floor(skipped),
			lastLaggedAt: new Date().toISOString(),
			updatedAt: new Date().toISOString()
		}
	}

	function resetRealtimeStatus(): void {
		status.value = { ...initialStatus }
	}

	function setReconnectHandler(handler: (() => void) | null): void {
		reconnectHandler = handler
	}

	function requestReconnect(): void {
		reconnectHandler?.()
	}

	return {
		status,
		isRealtimeDegraded,
		canTriggerReconnect,
		realtimeStatusLabel,
		realtimeStatusTone,
		realtimeStatusDetail,
		realtimeRecoveryDetail,
		setRealtimeStatus,
		observeRealtimeEvent,
		observeRealtimeLag,
		resetRealtimeStatus,
		setReconnectHandler,
		requestReconnect
	}
})

function formatRecoveryTimestamp(value: string): string {
	const date = new Date(value)
	if (Number.isNaN(date.getTime())) return value
	return date.toISOString()
}
