import type { Transport } from '@connectrpc/connect'
import { createConnectTransport } from '@connectrpc/connect-web'

import { BrowserGatewayFetch } from './browserGatewayFetch'
import type { BrowserGatewayFetchOptions } from './browserGatewayFetch'

export type BrowserGatewayConnectOptions = BrowserGatewayFetchOptions & {
	defaultTimeoutMs?: number
}

export const BROWSER_GATEWAY_REQUEST_TIMEOUT_MS = 10_000

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
	const {
		defaultTimeoutMs = BROWSER_GATEWAY_REQUEST_TIMEOUT_MS,
		...fetchOptions
	} = options
	if (
		!Number.isInteger(defaultTimeoutMs)
		|| defaultTimeoutMs < 1
		|| defaultTimeoutMs > BROWSER_GATEWAY_REQUEST_TIMEOUT_MS
	) {
		throw new RangeError('Browser Gateway timeout exceeds the admitted transport deadline')
	}
	const browserFetch = new BrowserGatewayFetch(fetchOptions)

	return createConnectTransport({
		baseUrl: '/',
		defaultTimeoutMs,
		fetch: browserFetch.fetch.bind(browserFetch),
		useBinaryFormat: true,
		useHttpGet: false,
	})
}
