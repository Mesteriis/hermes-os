import { ApiClient } from '$lib/api/client';

export const apiBaseUrl: string =
	import.meta.env.VITE_HERMES_API_BASE_URL ?? 'http://127.0.0.1:8080';

export const apiSecret: string =
	import.meta.env.VITE_HERMES_LOCAL_API_SECRET ?? 'change-me-local-api-secret';

ApiClient.init(apiBaseUrl, apiSecret);
