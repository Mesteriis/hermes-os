import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { createPinia, setActivePinia } from 'pinia'
import { useIntegrationConnectionWizardStore } from '../stores/integrationConnectionWizard'

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
    expect(component).toContain('<Steps')
    expect(component).toContain('<template #step-1>')
    expect(component).toContain('<template #step-2>')
    expect(component).toContain('<template #step-3>')
    expect(component).toContain('surface.handleSubmit')
    expect(component).toContain('surface.icloudPassword')
    expect(component).toContain("surface.telegramLoginMode.value === 'qr'")
    expect(component).toContain("surface.setTelegramLoginMode('phone')")
    expect(component).not.toContain('client_secret')
    expect(component).not.toContain('callback route')
    expect(component).not.toContain('session material')

    expect(surface).toContain('useIntegrationConnectionWizardStore')
    expect(surface).toContain('useStartGmailOAuthSetupMutation')
    expect(surface).toContain('useSetupImapEmailAccountMutation')
    expect(surface).toContain('startTelegramQrLogin')
    expect(surface).toContain('useStartHiddenWhatsappWebviewMutation')
    expect(surface).toContain('activeFlowIcon')
    expect(surface).toContain('handleSubmit')
    expect(surface).toContain('selectedProviderChecks')
    expect(surface).toContain('selectedProviderCapabilities')
    expect(surface).toContain('providerCards')
    expect(surface).toContain('providerStatus')
    expect(surface).toContain('exceptionFlowCard')
    expect(surface).toContain('telegramQrResult')
    expect(surface).toContain("export type TelegramLoginMode = 'qr' | 'phone'")
    expect(surface).toContain("const telegramLoginMode = ref<TelegramLoginMode>('qr')")
    expect(surface).toContain('setTelegramLoginMode')
    expect(surface).toContain('telegramQrErrorMessage')
    expect(surface).toContain('Не настроены Telegram API ID/API Hash.')
    expect(surface).toContain('setupIcloudConnection')
    expect(surface).toContain('wizard.buildGmailOAuthRequest(displayName, selectedAccount)')
    expect(component).toContain('surface.guidedQrImage')

    expect(store).toContain("'icloud'")
    expect(store).toContain('Ошибка подключения')
    expect(store).toContain('Browser callback')
    expect(store).toContain('QR companion')
    expect(store).toContain('Exception route')
    expect(store).toContain('previewProvider')
    expect(store).toContain('buildTelegramQrLoginRequest')
    expect(store).toContain('RecoverableConnectionAccount')
    expect(store).toContain('explicit exception handling')
    expect(store).toContain('buildGmailOAuthRequest')
    expect(store).toContain('app_return_url: gmailOAuthAppReturnUrl()')
  })

  it('returns to Settings after Gmail OAuth callback fallback redirects the current tab', () => {
    setActivePinia(createPinia())

    const store = useIntegrationConnectionWizardStore()
    const request = store.buildGmailOAuthRequest('Personal Gmail')

    expect(request.app_return_url).toBe(
      'http://127.0.0.1:5174/?hermes_route=settings&hermes_oauth=gmail_connected'
    )
  })
})
