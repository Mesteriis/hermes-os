import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiClient } from '../client';
import { toggleMessageImportant } from './communications';

describe('communications API endpoints', () => {
	beforeEach(() => {
		ApiClient.init('http://127.0.0.1:8080', 'local-secret');
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				new Response(JSON.stringify({ message_id: 'message primary', important: true }), {
					status: 200,
					headers: { 'Content-Type': 'application/json' }
				})
			)
		);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('toggles manual important flag through the dedicated message route', async () => {
		const result = await toggleMessageImportant('message primary');

		expect(result.important).toBe(true);
		const fetchMock = vi.mocked(fetch);
		expect(fetchMock.mock.calls[0][0]).toBe(
			'http://127.0.0.1:8080/api/v1/communications/messages/message%20primary/important'
		);
		expect(fetchMock.mock.calls[0][1]?.method).toBe('POST');
		expect(fetchMock.mock.calls[0][1]?.headers).toEqual({
			'Content-Type': 'application/json',
			'X-Hermes-Secret': 'local-secret'
		});
	});
});
