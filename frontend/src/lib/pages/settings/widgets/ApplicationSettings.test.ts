import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

describe('ApplicationSettings widget AI boundary', () => {
	it('does not render legacy ai.* fallback values as application runtime settings', () => {
		const source = readFileSync(new URL('./ApplicationSettings.svelte', import.meta.url), 'utf8');

		expect(source).not.toMatch(/settingValueTextFn\(['"]ai\./);
		expect(source).toContain('AI Control Center');
	});
});
