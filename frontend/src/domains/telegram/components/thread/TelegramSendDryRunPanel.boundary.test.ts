import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramSendDryRunPanel boundary', () => {
  it('consumes automation policies/templates and dry-run mutation through query hooks', () => {
    const source = readFileSync(new URL('./TelegramSendDryRunPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('useTelegramAutomationPoliciesQuery')
    expect(source).toContain('useTelegramAutomationTemplatesQuery')
    expect(source).toContain('useTelegramSendDryRunMutation')
    expect(source).toContain("t('Send Dry Run')")
    expect(source).toContain("t('Run Dry Run')")
    expect(source).not.toContain('fetch(')
  })
})
