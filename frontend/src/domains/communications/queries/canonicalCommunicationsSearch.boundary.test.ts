import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('canonical Communications search boundary', () => {
	it('uses the generated owner query client without provider or legacy API access', () => {
		const source = readFileSync(new URL('./canonicalCommunicationsSearch.ts', import.meta.url), 'utf8')

		expect(source).toContain('getCommunicationsQueryConnectClient')
		expect(source).toContain("case: 'searchCommunications'")
		expect(source).not.toMatch(/\.\.?\/api\//)
		expect(source).not.toContain('CommunicationsService')
		expect(source).not.toContain('provider')
	})
})
