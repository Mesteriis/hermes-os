import { describe, expect, it } from 'vitest';

import {
	buildGmailOAuthReturnUrl,
	isGmailOAuthConnectedSearch,
	isTrustedGmailOAuthConnectedMessage,
	removeHermesOAuthSearch
} from './oauth-callback';

describe('oauth callback helpers', () => {
	it('builds a return URL that reopens Hermes on the Gmail connected state', () => {
		expect(buildGmailOAuthReturnUrl('http://127.0.0.1:5174')).toBe(
			'http://127.0.0.1:5174/?hermes_oauth=gmail_connected'
		);
	});

	it('accepts only Gmail connected messages from the configured backend origin', () => {
		expect(
			isTrustedGmailOAuthConnectedMessage(
				{ origin: 'http://127.0.0.1:8080', data: { type: 'hermes:gmail-oauth-connected' } },
				'http://127.0.0.1:8080'
			)
		).toBe(true);
		expect(
			isTrustedGmailOAuthConnectedMessage(
				{ origin: 'http://evil.localhost', data: { type: 'hermes:gmail-oauth-connected' } },
				'http://127.0.0.1:8080'
			)
		).toBe(false);
	});

	it('detects and strips Gmail OAuth return query state', () => {
		const url = new URL('http://127.0.0.1:5174/?hermes_oauth=gmail_connected&keep=1#settings');

		expect(isGmailOAuthConnectedSearch(url.search)).toBe(true);
		expect(removeHermesOAuthSearch(url)).toBe('/?keep=1#settings');
	});
});
