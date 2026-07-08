import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('BackgroundJobsSettings boundary', () => {
  it('keeps Background Jobs under Settings and backed by observed runtime contracts', () => {
    const page = readFileSync(new URL('../views/SettingsPage.vue', import.meta.url), 'utf8')
    const settingsStore = readFileSync(new URL('../stores/settings.ts', import.meta.url), 'utf8')
    const pageSurface = readFileSync(new URL('../queries/useSettingsPageSurface.ts', import.meta.url), 'utf8')
    const jobsSurface = readFileSync(
      new URL('../queries/useBackgroundJobsSettingsSurface.ts', import.meta.url),
      'utf8'
    )
    const panel = readFileSync(new URL('./BackgroundJobsSettingsPanel.vue', import.meta.url), 'utf8')
    const presentation = readFileSync(new URL('./backgroundJobsPresentation.ts', import.meta.url), 'utf8')
    const main = readFileSync(new URL('../../../main.ts', import.meta.url), 'utf8')
    const bootstrap = readFileSync(
      new URL('../../../../../backend/src/application/bootstrap.rs', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./BackgroundJobsSettingsPanel.vue', import.meta.url))).toBe(true)
    expect(settingsStore).toContain("'background-jobs'")
    expect(pageSurface).toContain("id: 'background-jobs'")
    expect(pageSurface).toContain("label: 'Background Jobs'")
    expect(page).toContain('import BackgroundJobsSettingsPanel')
    expect(page).toContain("store.selectedSection === 'background-jobs'")
    expect(page).toContain(':surface="backgroundJobsSettings"')
    expect(main).toContain("import './styles/settings-background-jobs.css'")

    expect(jobsSurface).toContain('useSyncStatusesQuery')
    expect(jobsSurface).toContain('signalHubSettings.runtimeStates')
    expect(jobsSurface).toContain('signalHubSettings.replayPendingCount')
    expect(jobsSurface).toContain('realtimeStatus.realtimeStatusLabel')
    expect(jobsSurface).toContain('aiSettings.models')
    expect(jobsSurface).not.toContain('/api/v1/integrations/mail/accounts/sync-status')
    expect(jobsSurface).not.toContain('ApiClient')
    expect(jobsSurface).not.toMatch(/\bfetch\s*\(/)

    expect(panel).toContain('settings-background-tabs')
    expect(panel).toContain('settings-background-job-list')
    expect(panel).toContain('settings-background-mail-table')
    expect(panel).toContain('surface.handleOpenControl')

    for (const runtimeKind of [
      'mail_background_sync',
      'mail_outbox_delivery',
      'telegram_command_executor',
      'whatsapp_command_executor',
      'signal_hub_raw_signal_dispatcher',
      'signal_replay_dispatcher',
      'event_outbox_dispatcher'
    ]) {
      expect(presentation).toContain(runtimeKind)
      expect(bootstrap).toContain(`"${runtimeKind}"`)
    }
  })
})
