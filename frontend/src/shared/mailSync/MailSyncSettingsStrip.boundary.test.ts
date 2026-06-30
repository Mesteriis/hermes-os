import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('MailSyncSettingsStrip boundary', () => {
  it('preserves sync settings form contracts after removing legacy mail sync Vue strips', () => {
    const sharedFormSource = readFileSync(new URL('./syncSettingsForm.ts', import.meta.url), 'utf8')
    const integrationFormSource = readFileSync(
      new URL('../../integrations/mail/forms/syncSettingsForm.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./MailSyncSettingsStrip.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../../integrations/mail/components/MailSyncSettingsStrip.vue', import.meta.url))).toBe(false)

    for (const source of [sharedFormSource, integrationFormSource]) {
      expect(source).toContain('syncSettingsFormSchema')
      expect(source).toContain('syncSettingsVeeValidationSchema')
      expect(source).toContain('syncSettingsFormDefaults')
      expect(source).toContain('syncSettingsFormToUpdate')
      expect(source).toContain('batch_size')
      expect(source).toContain('poll_interval_seconds')
      expect(source).toContain('sync_enabled')
    }
  })
})
