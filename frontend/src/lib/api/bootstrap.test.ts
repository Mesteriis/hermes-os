import { describe, expect, it } from 'vitest';
import { ApiClient } from './index';

describe('API bootstrap', () => {
	it('initializes ApiClient when importing $lib/api', () => {
		expect(ApiClient.instance.baseUrl).toBe('http://127.0.0.1:8080');
	});
});
