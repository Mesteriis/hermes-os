import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { loadFrontendConfig } from '../../platform/config/env'
import type { GmailOAuthStartRequest } from '../../integrations/mail/api/accountSetup'
import type { TelegramQrLoginStartRequest } from '../integrationSetup/api/telegramQrLogin'

export type ConnectionProviderId =
  | 'mail'
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

const GMAIL_CONNECTION_ACCOUNT_PREFIX = 'mail-gmail'

const providerCatalog: ConnectionProviderOption[] = [
  {
    id: 'mail',
    label: 'Gmail',
    icon: 'tabler:mail',
    summary: 'Secure Google sign-in launches in the browser and returns to Hermes automatically.',
    flowLabel: 'Browser callback',
    entryLabel: 'Managed OAuth',
    status: 'Ready now',
    guidance: 'Mailbox passwords and client secrets never appear in Settings.',
    ctaLabel: 'Start Google sign-in',
    flowPattern: 'browser_callback',
  },
  {
    id: 'whatsapp',
    label: 'WhatsApp',
    icon: 'tabler:brand-whatsapp',
    summary: 'Visible companion runtime handles QR pairing and keeps pairing artifacts off the settings surface.',
    flowLabel: 'QR companion',
    entryLabel: 'Managed runtime',
    status: 'Resume from selected account',
    guidance: 'Open the managed companion window to continue QR pairing. Phone and session material stay off this surface.',
    ctaLabel: 'Open companion',
    flowPattern: 'qr_companion',
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
    id: 'zoom',
    label: 'Zoom',
    icon: 'tabler:video',
    summary: 'Zoom authorization returns through a browser callback after the workspace-managed app is provisioned.',
    flowLabel: 'Browser callback',
    entryLabel: 'Workspace callback',
    status: 'Workspace-prepared',
    guidance: 'Zoom client credentials are provisioned outside this screen. Manual secret entry stays disabled.',
    ctaLabel: 'Review callback route',
    flowPattern: 'browser_callback',
  },
  {
    id: 'yandex_telemost',
    label: 'Yandex Telemost',
    icon: 'tabler:video-plus',
    summary: 'Telemost connections follow a managed callback flow rather than an owner-entered credential form.',
    flowLabel: 'Browser callback',
    entryLabel: 'Workspace callback',
    status: 'Workspace-prepared',
    guidance: 'OAuth and runtime secrets are provisioned by the managed flow, not by ad hoc settings fields.',
    ctaLabel: 'Review callback route',
    flowPattern: 'browser_callback',
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
    case 'telegram_user':
    case 'telegram_bot':
      return 'telegram'
    case 'whatsapp_web':
    case 'whatsapp_business_cloud':
      return 'whatsapp'
    case 'zoom_user':
    case 'zoom_server_to_server':
      return 'zoom'
    case 'yandex_telemost_user':
      return 'yandex_telemost'
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
        title: 'Connection failed',
        message,
      }
    }

    function buildGmailOAuthRequest(): GmailOAuthStartRequest {
      const accountId = makeMailAccountId()

      return {
        account_id: accountId,
        display_name: 'Gmail',
        external_account_id: accountId,
        redirect_uri: gmailOAuthRedirectUri(),
      }
    }

    function buildTelegramQrLoginRequest(): TelegramQrLoginStartRequest {
      const accountId = makeTelegramAccountId()

      return {
        account_id: accountId,
        display_name: 'Telegram',
        external_account_id: accountId,
        transcription_enabled: true,
      }
    }

    function gmailOAuthRedirectUri(): string {
      return `${frontendConfig.apiBaseUrl.replace(/\/+$/, '')}/api/v1/integrations/mail/accounts/gmail/oauth/callback`
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
