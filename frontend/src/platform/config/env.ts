export type FrontendConfig = {
	apiBaseUrl: string
	apiSecret: string
	sseUrl: string
}

type EnvSource = Record<string, string | boolean | undefined>

const DEFAULT_API_BASE_URL = 'http://127.0.0.1:8080'

export function loadFrontendConfig(env: EnvSource = import.meta.env): FrontendConfig {
	const apiBaseUrl = normalizeBaseUrl(
		stringValue(env.VITE_HERMES_API_BASE_URL) ?? DEFAULT_API_BASE_URL
	)
	const apiSecret = stringValue(env.VITE_HERMES_LOCAL_API_SECRET)

	if (!apiSecret) {
		throw new Error('VITE_HERMES_LOCAL_API_SECRET is required')
	}

	return {
		apiBaseUrl,
		apiSecret,
		sseUrl: stringValue(env.VITE_HERMES_SSE_URL) ?? `${apiBaseUrl}/api/events/stream`
	}
}

function stringValue(value: string | boolean | undefined): string | undefined {
	return typeof value === 'string' && value.trim().length > 0 ? value.trim() : undefined
}

function normalizeBaseUrl(value: string): string {
	return value.replace(/\/+$/, '')
}
