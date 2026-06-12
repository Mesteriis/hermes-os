import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

const repoRoot = resolve(__dirname, '../../../..');

describe('vault onboarding shell layout contract', () => {
	it('keeps vault onboarding out of desktop shell grid at every viewport width', () => {
		const css = readFileSync(
			resolve(repoRoot, 'src/lib/components/vault/vault.css'),
			'utf8'
		).trimStart();
		const layout = readFileSync(resolve(repoRoot, 'src/routes/+layout.svelte'), 'utf8');

		expect(css.startsWith('.vault-onboarding')).toBe(true);
		expect(layout).toContain('{:else}');
		expect(layout.indexOf('<VaultOnboarding')).toBeLessThan(layout.indexOf('{:else}'));
		expect(layout.indexOf('{:else}')).toBeLessThan(layout.indexOf('<Sidebar'));
	});
});
