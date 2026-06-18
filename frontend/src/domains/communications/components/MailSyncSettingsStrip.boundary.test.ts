import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MailSyncSettingsStrip boundary', () => {
	it('renders provider sync settings controls without direct API access', () => {
		const source = readFileSync(new URL('./MailSyncSettingsStrip.vue', import.meta.url), 'utf8')

		expect(source).toContain('sync_enabled')
		expect(source).toContain('batch_size')
		expect(source).toContain('poll_interval_seconds')
		expect(source).toContain('Provider sync')
		expect(source).toContain('Save')
		expect(source).toContain('defineEmits')
		expect(source).toContain('update')
		expect(source).not.toContain('../api/')
		expect(source).not.toMatch(/\bfetch\s*\(/)
		expect(source).not.toContain('ApiClient')
	})
})
