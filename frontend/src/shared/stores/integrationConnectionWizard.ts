import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { loadFrontendConfig } from '../../platform/config/env'
import type { GmailOAuthStartRequest } from '../../integrations/mail/api/accountSetup'
import type { TelegramQrLoginStartRequest } from '../integrationSetup/api/telegramQrLogin'

export type ConnectionProviderId =
  | 'mail'
  | 'icloud'
  | 'telegram'
  | 'whatsapp'
  | 'zoom'
  | 'yandex_telemost'
  | 'zulip'

export type ConnectionWizardStep = 'provider' | 'details'
export type GuidedResultKind = 'blocked' | 'success' | 'error'
export type ConnectionFlowPattern = 'browser_callback' | 'qr_companion' | 'managed_surface'

export interface ConnectionProviderOption {
  id: ConnectionProviderId
  label: string
  icon: string
  summary: string
  flowLabel: string
  entryLabel: string
  status: string
  guidance: string
  ctaLabel: string
  flowPattern: ConnectionFlowPattern
}

export interface GuidedConnectionResult {
  kind: GuidedResultKind
  title: string
  message: string
  blockers?: string[]
  setupId?: string
  status?: string
  qrSvg?: string
  qrLink?: string
}

export interface RecoverableConnectionAccount {
  account_id: string
  external_account_id?: string | null
}

const GMAIL_CONNECTION_ACCOUNT_PREFIX = 'mail-gmail'
const DEFAULT_GMAIL_APP_RETURN_ORIGIN = 'http://127.0.0.1:5174'

const providerCatalog: ConnectionProviderOption[] = [
  {
    id: 'mail',
    label: 'Gmail',
    icon: 'simple-icons:gmail',
    summary: 'Secure Google sign-in launches in the browser and returns to Hermes automatically.',
    flowLabel: 'Browser callback',
    entryLabel: 'Managed OAuth',
    status: 'Ready now',
    guidance: 'Mailbox passwords and client secrets never appear in Settings.',
    ctaLabel: 'Start Google sign-in',
    flowPattern: 'browser_callback',
  },
  {
    id: 'icloud',
    label: 'iCloud Mail',
    icon: 'simple-icons:icloud',
    summary: 'iCloud Mail connects with the owner-entered email and app password.',
    flowLabel: 'Mail setup',
    entryLabel: 'iCloud Mail',
    status: 'Ready now',
    guidance: 'Use the iCloud mailbox address and app password.',
    ctaLabel: 'Connect iCloud',
    flowPattern: 'managed_surface',
  },
  {
    id: 'telegram',
    label: 'Telegram',
    icon: 'tabler:brand-telegram',
    summary: 'Telegram setup uses the backend TDLib QR login contract and keeps session material outside Settings.',
    flowLabel: 'QR companion',
    entryLabel: 'TDLib QR login',
    status: 'QR ready',
    guidance: 'Scan the QR code with Telegram. TDLib session material remains in the provider runtime boundary.',
    ctaLabel: 'Start Telegram QR',
    flowPattern: 'qr_companion',
  },
  {
    id: 'zulip',
    label: 'Zulip',
    icon: 'tabler:message-bolt',
    summary: 'Zulip bot setup continues in a dedicated runtime so secrets never leak into the generic settings surface.',
    flowLabel: 'Exception route',
    entryLabel: 'Dedicated runtime',
    status: 'Guided outside Settings',
    guidance: 'Bot credentials are provisioned outside this wizard. Manual key entry stays hidden behind explicit exception handling.',
    ctaLabel: 'View exception route',
    flowPattern: 'managed_surface',
  },
]

export function connectionProviderIdFromAccountKind(
  providerKind: string | null | undefined
): ConnectionProviderId {
  switch (providerKind) {
    case 'icloud':
      return 'icloud'
    case 'telegram_user':
    case 'telegram_bot':
      return 'telegram'
    case 'zulip_bot':
      return 'zulip'
    default:
      return 'mail'
  }
}

export const useIntegrationConnectionWizardStore = defineStore(
  'integration-connection-wizard',
  () => {
    const frontendConfig = loadFrontendConfig()
    const step = ref<ConnectionWizardStep>('provider')
    const selectedProviderId = ref<ConnectionProviderId>('mail')
    const errorMessage = ref('')
    const statusMessage = ref('')
    const guidedResult = ref<GuidedConnectionResult | null>(null)

    const selectedProvider = computed(() => {
      return providerCatalog.find((provider) => provider.id === selectedProviderId.value) ?? providerCatalog[0]
    })

    function reset(defaultProviderId: ConnectionProviderId = 'mail') {
      step.value = 'provider'
      selectedProviderId.value = defaultProviderId
      clearMessages()
    }

    function chooseProvider(providerId: ConnectionProviderId) {
      selectedProviderId.value = providerId
      step.value = 'details'
      clearMessages()
    }

    function previewProvider(providerId: ConnectionProviderId) {
      selectedProviderId.value = providerId
      clearMessages()
    }

    function goBack() {
      step.value = 'provider'
      clearMessages()
    }

    function close() {
      step.value = 'provider'
      clearMessages()
    }

    function clearMessages() {
      errorMessage.value = ''
      statusMessage.value = ''
      guidedResult.value = null
    }

    function setStatusMessage(message: string) {
      statusMessage.value = message
      errorMessage.value = ''
    }

    function setSuccess(result: Omit<GuidedConnectionResult, 'kind'>) {
      errorMessage.value = ''
      statusMessage.value = result.message
      guidedResult.value = {
        kind: 'success',
        ...result,
      }
    }

    function setBlocked(result: Omit<GuidedConnectionResult, 'kind'>) {
      errorMessage.value = ''
      statusMessage.value = ''
      guidedResult.value = {
        kind: 'blocked',
        ...result,
      }
    }

    function setError(message: string) {
      errorMessage.value = message
      statusMessage.value = ''
      guidedResult.value = {
        kind: 'error',
        title: 'Ошибка подключения',
        message,
      }
    }

    function buildGmailOAuthRequest(
      displayName = 'Gmail',
      account: RecoverableConnectionAccount | null = null
    ): GmailOAuthStartRequest {
      const accountId = account?.account_id.trim() || makeMailAccountId()
      const externalAccountId = account?.external_account_id?.trim() || accountId

      return {
        account_id: accountId,
        display_name: displayName.trim() || externalAccountId || 'Gmail',
        external_account_id: externalAccountId,
        redirect_uri: gmailOAuthRedirectUri(),
        app_return_url: gmailOAuthAppReturnUrl(),
      }
    }

    function buildTelegramQrLoginRequest(displayName = 'Telegram'): TelegramQrLoginStartRequest {
      const accountId = makeTelegramAccountId()

      return {
        account_id: accountId,
        display_name: displayName.trim() || 'Telegram',
        external_account_id: accountId,
        transcription_enabled: true,
      }
    }

    function gmailOAuthRedirectUri(): string {
      return `${frontendConfig.apiBaseUrl.replace(/\/+$/, '')}/api/v1/integrations/mail/accounts/gmail/oauth/callback`
    }

    function gmailOAuthAppReturnUrl(): string {
      const origin =
        typeof window === 'undefined'
          ? DEFAULT_GMAIL_APP_RETURN_ORIGIN
          : window.location.origin
      const returnUrl = new URL('/', origin)
      returnUrl.searchParams.set('hermes_route', 'settings')
      returnUrl.searchParams.set('hermes_oauth', 'gmail_connected')
      return returnUrl.toString()
    }

    function makeMailAccountId(): string {
      const nonce = `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`
      return `${GMAIL_CONNECTION_ACCOUNT_PREFIX}-${nonce}`
    }

    function makeTelegramAccountId(): string {
      const nonce = `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`
      return `telegram-qr-${nonce}`
    }

    return {
      providerCatalog,
      step,
      selectedProviderId,
      selectedProvider,
      errorMessage,
      statusMessage,
      guidedResult,
      reset,
      chooseProvider,
      previewProvider,
      goBack,
      close,
      clearMessages,
      setStatusMessage,
      setSuccess,
      setBlocked,
      setError,
      buildGmailOAuthRequest,
      buildTelegramQrLoginRequest,
    }
  }
)
