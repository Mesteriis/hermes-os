import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MailSyncSettingsStrip boundary', () => {
	it('renders provider sync settings controls without direct API access', () => {
		const source = readFileSync(new URL('./MailSyncSettingsStrip.vue', import.meta.url), 'utf8')

		expect(source).toContain('syncEnabled')
		expect(source).toContain('batchSize')
		expect(source).toContain('pollIntervalSeconds')
		expect(source).toContain('Provider sync')
		expect(source).toContain('Save')
		expect(source).toContain('defineEmits')
		expect(source).toContain('update')
		expect(source).not.toContain('../api/')
		expect(source).not.toContain('fetch(')
		expect(source).not.toContain('ApiClient')
	})
})
