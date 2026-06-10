export const GMAIL_OAUTH_CONNECTED_MESSAGE = 'hermes:gmail-oauth-connected';
export const GMAIL_OAUTH_CONNECTED_QUERY_VALUE = 'gmail_connected';

export function buildGmailOAuthReturnUrl(origin: string): string {
	const url = new URL(origin);
	url.searchParams.set('hermes_oauth', GMAIL_OAUTH_CONNECTED_QUERY_VALUE);
	return url.toString();
}

export function isGmailOAuthConnectedSearch(search: string): boolean {
	return new URLSearchParams(search).get('hermes_oauth') === GMAIL_OAUTH_CONNECTED_QUERY_VALUE;
}

export function isTrustedGmailOAuthConnectedMessage(
	event: Pick<MessageEvent, 'origin' | 'data'>,
	apiBaseUrl: string
): boolean {
	let apiOrigin: string;
	try {
		apiOrigin = new URL(apiBaseUrl).origin;
	} catch {
		return false;
	}

	return (
		event.origin === apiOrigin &&
		typeof event.data === 'object' &&
		event.data !== null &&
		(event.data as { type?: unknown }).type === GMAIL_OAUTH_CONNECTED_MESSAGE
	);
}

export function removeHermesOAuthSearch(url: URL): string {
	const next = new URL(url);
	next.searchParams.delete('hermes_oauth');
	const search = next.searchParams.toString();
	return `${next.pathname}${search ? `?${search}` : ''}${next.hash}`;
}
