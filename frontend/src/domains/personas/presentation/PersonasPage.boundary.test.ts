import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('PersonasPage clean presentation boundary', () => {
	it('uses only the typed page model and existing UI workspace', () => {
		const source = readFileSync(new URL('./PersonasPage.vue', import.meta.url), 'utf8')

		expect(source).toContain("import PersonasWorkspace from '../components/PersonasWorkspace.vue'")
		expect(source).toContain("import type { PersonasPageActions, PersonasPageModel } from './personasPageModel'")
		expect(source).not.toMatch(/\/api\/|\/queries\/|\/stores\/|ApiClient|fetch\(/)
	})
})
