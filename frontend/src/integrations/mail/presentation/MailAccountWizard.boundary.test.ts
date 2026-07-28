import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Mail account wizard composition', () => {
	it('keeps provider setup account-scoped and credential custody outside Settings', () => {
		const wizard = read('./MailAccountSetupPanel.vue')
		const management = read('./MailAccountManagementPanel.vue')
		const setup = read('../setup/mailAccountSetupWorkflow.ts')
		const catalog = read('../api/mailAccountQueryClient.ts')

		expect(wizard).toContain('<Steps')
		expect(wizard).toContain('Gmail')
		expect(wizard).toContain('iCloud Mail')
		expect(wizard).toContain('Custom IMAP')
		expect(wizard).toContain('setup.gmailState.value.started.authorizationUrl')
		expect(management).toContain('management.accounts.value')
		expect(management).toContain('management.selectAccount')
		expect(setup).toContain('createTarget')
		expect(setup).toContain('configurationInstanceId')
		expect(catalog).toContain('MailAccountCatalogService')
		expect(catalog).toContain('{ major: 1 }')
		for (const source of [wizard, management, setup, catalog]) {
			expect(source).not.toMatch(/domains\/communications/)
			expect(source).not.toMatch(/password.*settingId|settingId.*password/i)
		}
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
