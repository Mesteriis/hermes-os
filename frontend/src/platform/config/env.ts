export type FrontendConfig = {
	apiBaseUrl: string
	apiSecret: string
	sseUrl: string
	webSocketUrl: string
	realtimeTransport: RealtimeTransportPreference
}

export type RealtimeTransportPreference = 'websocket' | 'sse'
type EnvSource = Record<string, string | boolean | undefined>

const DEFAULT_API_BASE_URL = 'http://127.0.0.1:8080'
// Must match the backend DEFAULT_LOCAL_API_SECRET fallback so `make dev` and
// plain `pnpm dev` work without any environment setup. Packaged desktop builds
// bake a per-build random secret in via VITE_HERMES_LOCAL_API_SECRET.
const DEFAULT_LOCAL_API_SECRET = ['change-me', 'local-api', 'secret'].join('-')

export function loadFrontendConfig(env: EnvSource = import.meta.env): FrontendConfig {
	const apiBaseUrl = normalizeBaseUrl(
		stringValue(env.VITE_HERMES_API_BASE_URL) ?? DEFAULT_API_BASE_URL
	)
	const apiSecret = stringValue(env.VITE_HERMES_LOCAL_API_SECRET) ?? DEFAULT_LOCAL_API_SECRET

	return {
		apiBaseUrl,
		apiSecret,
		sseUrl: stringValue(env.VITE_HERMES_SSE_URL) ?? `${apiBaseUrl}/api/events/stream`,
		webSocketUrl:
			stringValue(env.VITE_HERMES_WEBSOCKET_URL) ?? defaultWebSocketUrl(apiBaseUrl),
		realtimeTransport: realtimeTransportPreference(env.VITE_HERMES_REALTIME_TRANSPORT)
	}
}

function stringValue(value: string | boolean | undefined): string | undefined {
	return typeof value === 'string' && value.trim().length > 0 ? value.trim() : undefined
}

function normalizeBaseUrl(value: string): string {
	return value.replace(/\/+$/, '')
}

function defaultWebSocketUrl(apiBaseUrl: string): string {
	const parsed = new URL(apiBaseUrl)
	parsed.protocol = parsed.protocol === 'https:' ? 'wss:' : 'ws:'
	parsed.pathname = '/api/events/ws'
	parsed.search = ''
	parsed.hash = ''
	return parsed.toString().replace(/\/$/, '')
}

function realtimeTransportPreference(
	value: string | boolean | undefined
): RealtimeTransportPreference {
	const normalized = stringValue(value)?.toLowerCase()
	if (normalized === 'websocket') return 'websocket'
	return 'sse'
}
