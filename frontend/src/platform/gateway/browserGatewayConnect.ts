import type { Transport } from '@connectrpc/connect'
import { createConnectTransport } from '@connectrpc/connect-web'

import { BrowserGatewayFetch } from './browserGatewayFetch'
import type { BrowserGatewayFetchOptions } from './browserGatewayFetch'

export type BrowserGatewayConnectOptions = BrowserGatewayFetchOptions & {
	defaultTimeoutMs?: number
}

/**
 * Typed Connect transport for the future browser-facing Core Gateway.
 *
 * It intentionally exposes no URL, headers, interceptors, or business methods:
 * every call is constrained by BrowserGatewayFetch to the current origin and
 * authenticated only by the Gateway's HttpOnly session cookie.
 */
export function createBrowserGatewayConnectTransport(
	options: BrowserGatewayConnectOptions = {},
): Transport {
	const { defaultTimeoutMs, ...fetchOptions } = options
	const browserFetch = new BrowserGatewayFetch(fetchOptions)

	return createConnectTransport({
		baseUrl: '/',
		defaultTimeoutMs,
		fetch: browserFetch.fetch.bind(browserFetch),
		useBinaryFormat: true,
		useHttpGet: false,
	})
}
