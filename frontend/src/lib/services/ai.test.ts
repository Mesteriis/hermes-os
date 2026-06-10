import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { refreshTasksFromAi } from './ai';

describe('AI service', () => {
	beforeEach(() => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				new Response(
					JSON.stringify({
						run_id: 'run-1',
						agent_id: 'MNEMOSYNE',
						status: 'completed',
						created_count: 3,
						citations: [],
						model: 'llama',
						embedding_model: 'nomic',
						created_at: '2026-06-09T12:00:00Z',
						duration_ms: 42
					}),
					{ status: 200, headers: { 'Content-Type': 'application/json' } }
				)
			)
		);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('refreshes task candidates through the dedicated endpoint, not the AI answer endpoint', async () => {
		const result = await refreshTasksFromAi('find tasks', 'ATHENA');

		expect(result.error).toBe('');
		expect(result.result?.created_count).toBe(3);

		const fetchMock = vi.mocked(fetch);
		expect(fetchMock).toHaveBeenCalledTimes(1);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe('http://127.0.0.1:8080/api/v1/ai/task-candidates/refresh');
		expect(url).not.toContain('/api/v1/ai/answers');
		expect(init?.method).toBe('POST');
		expect(JSON.parse(String(init?.body))).toMatchObject({ query: 'find tasks' });
	});
});
