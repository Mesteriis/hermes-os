import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Telegram automation settings boundary', () => {
	it('keeps generated query, command, controller and presentation responsibilities separate', () => {
		const route = read('./TelegramAutomationSettingsRoute.vue')
		const controller = read('../queries/useTelegramAutomationManagement.ts')
		const presentation = read('../presentation/TelegramAutomationPanel.vue')
		const model = read('../presentation/telegramAutomationModel.ts')
		const queryClient = read('../api/telegramAutomationQueryClient.ts')
		const commandClient = read('../api/telegramAutomationCommandClient.ts')
		const queryGateway = read('../api/telegramAutomationQueryGateway.ts')
		const commandGateway = read('../api/telegramAutomationCommandGateway.ts')
		const settingsComposition = read('../../../app/settings/AppSettingsPage.vue')
		const generator = read('../../../../scripts/generate-proto.mjs')

		for (const source of [
			route,
			controller,
			presentation,
			model,
			queryClient,
			commandClient,
			queryGateway,
			commandGateway,
		]) {
			expect(source).not.toMatch(/\/api\/v1\//)
			expect(source).not.toMatch(/domains\/communications/)
			expect(source).not.toMatch(/integrations\/(mail|whatsapp|zulip)/)
		}
		expect(queryClient).toContain('TelegramAutomationQueryService')
		expect(commandClient).toContain('TelegramAutomationCommandService')
		expect(queryGateway).toContain("case: 'listTemplates'")
		expect(queryGateway).toContain("case: 'listPolicies'")
		expect(commandGateway).toContain("case: 'upsertTemplate'")
		expect(commandGateway).toContain("case: 'upsertPolicy'")
		expect(commandGateway).toContain("case: 'previewPolicy'")
		expect(queryGateway).not.toMatch(/as never|unknown as|Record</)
		expect(commandGateway).not.toMatch(/as never|unknown as|Record</)
		expect(presentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(model).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(route).toContain('useTelegramAutomationManagement')
		expect(settingsComposition).toContain('TelegramAutomationSettingsRoute')
		expect(settingsComposition).toContain("'telegram.automation.command.v1'")
		expect(settingsComposition).toContain("'telegram.automation.query.v1'")
		expect(generator).toContain('telegram-automation-api')
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
