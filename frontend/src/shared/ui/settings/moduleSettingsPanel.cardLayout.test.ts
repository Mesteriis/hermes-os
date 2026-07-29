import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('module settings card layout', () => {
	it('keeps the owner panel and its groups as separated cards', () => {
		const styles = readFileSync(
			new URL('./moduleSettingsPanel.css', import.meta.url),
			'utf8',
		)

		expect(styles).toMatch(
			/\.module-settings-panel\s*\{[^}]*gap:\s*var\(--h-space-4\)[^}]*padding:\s*var\(--h-space-5\)[^}]*border:\s*1px solid var\(--h-color-border\)[^}]*border-radius:\s*var\(--h-radius-lg\)[^}]*background:[^}]*var\(--h-color-bg-muted\)[^}]*box-shadow:\s*var\(--h-shadow-sm\)/s,
		)
		expect(styles).toMatch(
			/\.module-settings-panel__header\s*\{[^}]*border:\s*1px solid var\(--h-color-border-strong\)[^}]*background:[^}]*var\(--h-color-surface-raised\)/s,
		)
		expect(styles).toMatch(
			/\.module-settings-panel__metadata > span\s*\{[^}]*background:\s*var\(--h-color-surface\)[^}]*box-shadow:\s*var\(--h-shadow-xs\)/s,
		)
		expect(styles).toMatch(
			/\.module-settings-panel__list\s*\{[^}]*gap:\s*var\(--h-space-3\)/s,
		)
	})
})
