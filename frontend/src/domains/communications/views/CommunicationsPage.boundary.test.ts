import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('CommunicationsPage boundary', () => {
	it('keeps the typed presentation page free from runtime orchestration', () => {
		const presentationSource = readFileSync(
			new URL('../presentation/CommunicationsPage.vue', import.meta.url),
			'utf8'
		)

		expect(existsSync(new URL('../presentation/CommunicationsPage.vue', import.meta.url))).toBe(true)
		expect(presentationSource).toContain('CommunicationsPageModel')
		expect(presentationSource).toContain('CommunicationsPageActions')
		expect(presentationSource).not.toMatch(/\/api\/|\/queries\/|\/stores\/|ApiClient|fetch\(/)
  })
})
