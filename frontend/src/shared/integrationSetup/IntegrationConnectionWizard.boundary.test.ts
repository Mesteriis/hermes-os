import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('IntegrationConnectionWizard boundary', () => {
  it('renders the wizard shell while keeping provider orchestration in store + mutation surfaces', () => {
    const component = readFileSync(
      new URL('./IntegrationConnectionWizard.vue', import.meta.url),
      'utf8'
    )
    const surface = readFileSync(
      new URL('./queries/useIntegrationConnectionWizardSurface.ts', import.meta.url),
      'utf8'
    )
    const store = readFileSync(
      new URL('../stores/integrationConnectionWizard.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./IntegrationConnectionWizard.vue', import.meta.url))).toBe(true)
    expect(existsSync(new URL('./IntegrationConnectionDetails.vue', import.meta.url))).toBe(false)

    expect(component).toContain('useIntegrationConnectionWizardSurface')
    expect(component).toContain('role="dialog"')
    expect(component).toContain('surface.handleSubmit')
    expect(component).not.toContain('password')
    expect(component).not.toContain('client_secret')

    expect(surface).toContain('useIntegrationConnectionWizardStore')
    expect(surface).toContain('useStartGmailOAuthSetupMutation')
    expect(surface).toContain('startTelegramQrLogin')
    expect(surface).toContain('useOpenWhatsappWebCompanionMutation')
    expect(surface).toContain('activeFlowIcon')
    expect(surface).toContain('handleSubmit')
    expect(surface).toContain('launchSteps')
    expect(surface).toContain('providerStatus')
    expect(surface).toContain('exceptionFlowCard')
    expect(surface).toContain('telegramQrResult')
    expect(component).toContain('surface.guidedQrImage')

    expect(store).toContain('Browser callback')
    expect(store).toContain('QR companion')
    expect(store).toContain('Exception route')
    expect(store).toContain('previewProvider')
    expect(store).toContain('buildTelegramQrLoginRequest')
    expect(store).toContain('explicit exception handling')
    expect(store).toContain('buildGmailOAuthRequest')
  })
})
