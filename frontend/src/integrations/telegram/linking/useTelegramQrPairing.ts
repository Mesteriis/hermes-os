import { computed, onBeforeUnmount, ref } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import {
	getTelegramAuthorizationStatus,
	submitTelegramAuthorizationPassword,
} from '../api/telegramAuthorizationGateway'
import { telegramQrDataUrl } from './telegramQrArtifact'

const TELEGRAM_AUTHORIZATION_CAPABILITY_ID = 'telegram.authorization.v1'
const AUTHORIZATION_POLL_INTERVAL_MS = 2_000
const TERMINAL_AUTHORIZATION_STATES = new Set(['ready', 'closed', 'error'])

export function useTelegramQrPairing(module: () => ClientModuleBootstrapV1 | null) {
	const state = ref('unknown')
	const qrDataUrl = ref('')
	const passwordHint = ref('')
	const password = ref('')
	const busy = ref(false)
	const message = ref('')
	const messageTone = ref<'neutral' | 'success' | 'error'>('neutral')
	let pollTimer: ReturnType<typeof setTimeout> | null = null
	const admitted = computed(
		() => module()?.capabilityIds.includes(TELEGRAM_AUTHORIZATION_CAPABILITY_ID) ?? false,
	)
	const configured = computed(() => (module()?.settings?.effectiveRevision ?? 0n) > 0n)
	const canRefresh = computed(() => admitted.value && configured.value)

	async function refresh(): Promise<void> {
		if (!canRefresh.value || busy.value) return
		busy.value = true
		try {
			const status = await getTelegramAuthorizationStatus()
			state.value = status.state || 'unknown'
			passwordHint.value = status.passwordHint ?? ''
			qrDataUrl.value = status.qrLink
				? await telegramQrDataUrl(status.qrLink)
				: ''
			message.value = statusMessage(state.value)
			messageTone.value = state.value === 'ready' ? 'success' : 'neutral'
			if (!TERMINAL_AUTHORIZATION_STATES.has(state.value)) scheduleRefresh()
		} catch {
			clearQr()
			message.value = 'Telegram authorization status is unavailable.'
			messageTone.value = 'error'
		} finally {
			busy.value = false
		}
	}

	async function submitPassword(): Promise<void> {
		if (!password.value.trim()) return
		busy.value = true
		try {
			await submitTelegramAuthorizationPassword(password.value)
			password.value = ''
			message.value = 'Telegram 2FA password accepted. Waiting for authorization.'
			messageTone.value = 'neutral'
		} catch {
			password.value = ''
			message.value = 'Telegram rejected the 2FA continuation.'
			messageTone.value = 'error'
		} finally {
			busy.value = false
		}
		await refresh()
	}

	function scheduleRefresh(): void {
		stopPolling()
		pollTimer = setTimeout(() => void refresh(), AUTHORIZATION_POLL_INTERVAL_MS)
	}

	function stopPolling(): void {
		if (pollTimer) clearTimeout(pollTimer)
		pollTimer = null
	}

	function clearQr(): void {
		qrDataUrl.value = ''
	}

	onBeforeUnmount(() => {
		stopPolling()
		clearQr()
		password.value = ''
	})

	return {
		state,
		qrDataUrl,
		passwordHint,
		password,
		busy,
		message,
		messageTone,
		admitted,
		configured,
		canRefresh,
		refresh,
		submitPassword,
	}
}

function statusMessage(state: string): string {
	switch (state) {
		case 'waiting_qr_scan':
			return 'Scan this QR code from Telegram → Settings → Devices → Link Desktop Device.'
		case 'waiting_password':
			return 'Telegram requires the account 2FA password to finish linking.'
		case 'ready':
			return 'Telegram account is authorized.'
		case 'error':
		case 'closed':
			return 'Telegram authorization stopped before completion.'
		default:
			return 'Preparing a provider-issued Telegram QR code.'
	}
}
