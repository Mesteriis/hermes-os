import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('IntegrationConnectionWizard boundary', () => {
  it('keeps connection orchestration in store + mutation surfaces after deleting the wizard Vue layer', () => {
    const surface = readFileSync(
      new URL('./queries/useIntegrationConnectionWizardSurface.ts', import.meta.url),
      'utf8'
    )
    const store = readFileSync(
      new URL('../stores/integrationConnectionWizard.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./IntegrationConnectionWizard.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./IntegrationConnectionDetails.vue', import.meta.url))).toBe(false)

    expect(surface).toContain('useIntegrationConnectionWizardStore')
    expect(surface).toContain('useStartGmailOAuthSetupMutation')
    expect(surface).toContain('useOpenWhatsappWebCompanionMutation')
    expect(surface).toContain('activeFlowIcon')
    expect(surface).toContain('handleSubmit')
    expect(surface).toContain('launchSteps')
    expect(surface).toContain('providerStatus')
    expect(surface).toContain('exceptionFlowCard')
    expect(surface).toContain('Exception route required')

    expect(store).toContain('Browser callback')
    expect(store).toContain('QR companion')
    expect(store).toContain('Exception route')
    expect(store).toContain('previewProvider')
    expect(store).toContain('explicit exception handling')
    expect(store).toContain('buildGmailOAuthRequest')
  })
})
