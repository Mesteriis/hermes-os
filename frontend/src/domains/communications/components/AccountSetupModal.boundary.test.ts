import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('AccountSetupModal account setup boundary', () => {
	it('uses validated account setup helpers and real API calls instead of simulated success', () => {
		const source = readFileSync(
			new URL('./AccountSetupModal.vue', import.meta.url),
			'utf8'
		)

		expect(source).toContain("from 'vee-validate'")
		expect(source).toContain('../forms/accountSetupForm')
		expect(source).toContain('../api/accountSetup')
		expect(source).toContain('setupImapEmailAccount')
		expect(source).toContain('startGmailOAuthSetup')
		expect(source).not.toContain('setTimeout')
	})
})
