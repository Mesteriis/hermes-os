/**
 * Application configuration read from environment / Vite import.meta.env.
 */
export const config = {
	/** Backend API base URL */
	apiBaseUrl: import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:3000',

	/** Hermes local API shared secret */
	apiSecret: import.meta.env.VITE_HERMES_API_SECRET ?? '',

	/** SSE endpoint URL */
	sseUrl: import.meta.env.VITE_SSE_URL ?? 'http://localhost:3001'
} as const
