import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('canonical communications card layout', () => {
	it('keeps the owner page and each workbench pane on separated cards', () => {
		const styles = readFileSync(
			new URL('./canonicalCommunicationsPage.css', import.meta.url),
			'utf8',
		)
		const route = readFileSync(
			new URL('../views/CanonicalCommunicationsRoute.vue', import.meta.url),
			'utf8',
		)

		expect(route).toContain('class="canonical-communications-route"')
		expect(styles).toContain('.canonical-communications-route')
		expect(styles).toContain('.canonical-communications-page__header')
		expect(styles).toContain('.canonical-communications-workbench')
		expect(styles).toContain('gap: var(--h-space-3)')
		expect(styles).toContain('height: min(46rem, calc(100dvh - 14rem))')
		expect(styles).toContain('.canonical-communications-pane')
		expect(styles).toContain('.canonical-communications-row')
		expect(styles).toContain('box-shadow: var(--h-shadow-xs)')
		expect(styles).toContain('border-radius: var(--h-radius-lg)')
		expect(styles).not.toContain('.canonical-communications-pane + .canonical-communications-pane')
	})
})
