import { computed, ref, toValue, watch, type MaybeRefOrGetter } from 'vue'
import { useMutation } from '@tanstack/vue-query'
import { useStartGmailOAuthSetupMutation } from '../../../integrations/mail/queries/accountSetupQueries'
import { useOpenWhatsappWebCompanionMutation } from '../../../integrations/whatsapp/queries/useWhatsappRuntimeQuery'
import { useI18n } from '../../../platform/i18n'
import {
  fetchTelegramQrLoginStatus,
  startTelegramQrLogin,
  type TelegramQrLoginStatusResponse,
} from '../api/telegramQrLogin'
import {
  connectionProviderIdFromAccountKind,
  useIntegrationConnectionWizardStore,
} from '../../stores/integrationConnectionWizard'
import type {
  ConnectionFlowPattern,
  ConnectionProviderId,
  ConnectionProviderOption,
} from '../../stores/integrationConnectionWizard'

export interface SelectedIntegrationAccount {
  account_id: string
  provider_kind: string
  display_name?: string | null
  external_account_id: string
}

export interface LaunchStep {
  title: string
  detail: string
}

interface ConnectionFlowCard {
  id: ConnectionFlowPattern
  label: string
  icon: string
  summary: string
  promise: string
  recovery: string
  providers: ConnectionProviderOption[]
}

const FLOW_CATALOG: Array<Omit<ConnectionFlowCard, 'providers'>> = [
  {
    id: 'browser_callback',
    label: 'Browser callback',
    icon: 'tabler:browser-share',
    summary: 'Launch provider auth in a secure browser tab and return to Hermes automatically.',
    promise: 'Primary route for browser-authorized providers',
    recovery: 'OAuth fields stay out of Settings.',
  },
  {
    id: 'qr_companion',
    label: 'QR companion',
    icon: 'tabler:qrcode',
    summary: 'Open a visible pairing runtime, scan the QR code from your phone and finish the link outside the settings surface.',
    promise: 'Visible device pairing',
    recovery: 'Phone and session material never render here.',
  },
  {
    id: 'managed_surface',
    label: 'Exception route',
    icon: 'tabler:route-alt-left',
    summary: 'Shown only when a provider cannot complete callback or QR onboarding from Settings.',
    promise: 'Hidden unless required',
    recovery: 'Manual recovery stays explicit and exceptional.',
  },
]

const FLOW_ICONS: Record<ConnectionFlowPattern, string> = {
  browser_callback: 'tabler:browser-share',
  qr_companion: 'tabler:qrcode',
  managed_surface: 'tabler:route-alt-left',
}

export function useIntegrationConnectionWizardSurface(options: {
  selectedAccount: MaybeRefOrGetter<SelectedIntegrationAccount | null | undefined>
  defaultProviderId: MaybeRefOrGetter<ConnectionProviderId | null | undefined>
}) {
  const { t } = useI18n()
  const wizard = useIntegrationConnectionWizardStore()
  const gmailOAuthSetup = useStartGmailOAuthSetupMutation()
  const telegramQrLogin = useMutation({ mutationFn: startTelegramQrLogin })
  const telegramQrStatus = useMutation({ mutationFn: fetchTelegramQrLoginStatus })
  const openCompanionMutation = useOpenWhatsappWebCompanionMutation()

  const activeFlowId = ref<ConnectionFlowPattern>('browser_callback')

  watch(
    () => [
      toValue(options.selectedAccount)?.provider_kind ?? null,
      toValue(options.defaultProviderId) ?? null,
    ] as const,
    ([providerKind, defaultProviderId]) => {
      const providerId = providerKind
        ? connectionProviderIdFromAccountKind(providerKind)
        : (defaultProviderId ?? 'mail')
      const provider =
        wizard.providerCatalog.find((item) => item.id === providerId) ?? wizard.providerCatalog[0]
      wizard.reset(provider.id)
      activeFlowId.value = provider.flowPattern
    },
    { immediate: true }
  )

  const selectedAccountId = computed(() => toValue(options.selectedAccount)?.account_id?.trim() || null)
  const selectedAccountLabel = computed(() => {
    const account = toValue(options.selectedAccount)
    if (!account) return null
    return (
      account.display_name?.trim() ||
      account.external_account_id?.trim() ||
      account.account_id
    )
  })
  const isSelectedWhatsAppAccount = computed(
    () => toValue(options.selectedAccount)?.provider_kind === 'whatsapp_web'
  )
  const flowCards = computed<ConnectionFlowCard[]>(() =>
    FLOW_CATALOG.map((flow) => ({
      ...flow,
      providers: wizard.providerCatalog.filter((provider) => provider.flowPattern === flow.id),
    }))
  )
  const primaryFlowCards = computed(() => flowCards.value.filter((flow) => flow.id !== 'managed_surface'))
  const exceptionFlowCard = computed(
    () => flowCards.value.find((flow) => flow.id === 'managed_surface') ?? null
  )
  const activeFlow = computed(
    () => flowCards.value.find((flow) => flow.id === activeFlowId.value) ?? flowCards.value[0]
  )
  const selectedProvider = computed(() => wizard.selectedProvider)
  const activeFlowIcon = computed(
    () => FLOW_ICONS[selectedProvider.value.flowPattern] ?? 'tabler:plug-connected'
  )
  const launchSteps = computed<LaunchStep[]>(() => {
    switch (selectedProvider.value.flowPattern) {
      case 'browser_callback':
        return [
          {
            title: 'Launch secure browser flow',
            detail: 'Hermes opens the managed provider authorization window.',
          },
          {
            title: 'Approve access at the provider',
            detail: 'Authentication happens outside the Settings surface.',
          },
          {
            title: 'Return automatically',
            detail: 'Hermes continues after the callback returns to the local workspace.',
          },
        ]
      case 'qr_companion':
        return [
          {
            title: 'Open visible companion',
            detail: 'The runtime opens in a dedicated window instead of embedding pairing secrets here.',
          },
          {
            title: 'Scan the QR code',
            detail: 'Use your phone to authorize the session in the provider-native flow.',
          },
          {
            title: 'Finish linking in runtime',
            detail: 'Hermes resumes when the companion confirms the pairing.',
          },
        ]
      default:
        return [
          {
            title: 'Escalate into an exception route',
            detail: 'This provider stays on a dedicated runtime route rather than a self-serve callback, QR flow or raw form.',
          },
          {
            title: 'Follow the guided runtime handoff',
            detail: 'Secrets and provider runtime material remain outside the generic settings panel.',
          },
          {
            title: 'Return when the runtime is ready',
            detail: 'Hermes shows the account only after the managed setup flow completes.',
          },
        ]
    }
  })
  const isSubmitting = computed(
    () => gmailOAuthSetup.isPending.value || telegramQrLogin.isPending.value || openCompanionMutation.isPending.value
  )
  const canSubmit = computed(() => {
    switch (selectedProvider.value.id) {
      case 'mail':
        return !gmailOAuthSetup.isPending.value
      case 'telegram':
        return !telegramQrLogin.isPending.value
      case 'whatsapp':
        return isSelectedWhatsAppAccount.value && !openCompanionMutation.isPending.value
      default:
        return true
    }
  })
  const submitLabel = computed(() => {
    if (selectedProvider.value.id === 'mail' && gmailOAuthSetup.isPending.value) {
      return t('Opening browser callback...')
    }
    if (selectedProvider.value.id === 'whatsapp' && openCompanionMutation.isPending.value) {
      return t('Opening QR companion...')
    }
    if (selectedProvider.value.id === 'telegram' && telegramQrLogin.isPending.value) {
      return t('Starting Telegram QR...')
    }
    return t(selectedProvider.value.ctaLabel)
  })
  const guidedQrImage = computed(() => {
    const qrSvg = wizard.guidedResult?.qrSvg
    if (!qrSvg) return null
    return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(qrSvg)}`
  })
  const canRefreshGuidedResult = computed(() =>
    selectedProvider.value.id === 'telegram' &&
    Boolean(wizard.guidedResult?.setupId) &&
    !telegramQrStatus.isPending.value
  )

  function providerStatus(providerId: string): string {
    if (providerId === 'whatsapp' && isSelectedWhatsAppAccount.value) {
      return t('QR ready from selected account')
    }
    return t(
      wizard.providerCatalog.find((provider) => provider.id === providerId)?.status ?? 'Managed only'
    )
  }

  function setActiveFlow(flowId: ConnectionFlowPattern) {
    activeFlowId.value = flowId
    const provider = wizard.providerCatalog.find((item) => item.flowPattern === flowId)
    if (provider) {
      wizard.previewProvider(provider.id)
    }
  }

  async function startManagedMailConnection() {
    try {
      const response = await gmailOAuthSetup.mutateAsync(wizard.buildGmailOAuthRequest())
      if (typeof window !== 'undefined' && typeof window.open === 'function') {
        window.open(response.authorization_url, '_blank', 'noopener,noreferrer')
      }
      wizard.setSuccess({
        title: t('Browser callback started'),
        message: t(
          'Google sign-in opened in a secure browser window. Hermes will continue after the callback returns.'
        ),
      })
    } catch (error) {
      wizard.setError(error instanceof Error ? error.message : t('Failed to start Google OAuth.'))
    }
  }

  async function openManagedWhatsappCompanion() {
    if (!selectedAccountId.value || !isSelectedWhatsAppAccount.value) {
      wizard.setBlocked({
        title: t('QR flow needs a WhatsApp account'),
        message: t('Select a WhatsApp Web account first, then reopen the wizard to resume QR pairing.'),
        blockers: [
          t('QR pairing launches only from a managed runtime account shell.'),
          t('Manual phone, cookie or session input stays disabled in Settings.'),
        ],
      })
      return
    }

    try {
      const manifest = await openCompanionMutation.mutateAsync({ account_id: selectedAccountId.value })
      wizard.setSuccess({
        title: t('QR companion ready'),
        message: manifest.focused_existing_window
          ? t('Existing companion window focused. Continue QR pairing there.')
          : t('Visible companion window opened. Scan the QR code there to finish linking.'),
      })
    } catch (error) {
      wizard.setError(
        error instanceof Error ? error.message : t('Failed to open the WhatsApp companion window.')
      )
    }
  }

  async function startManagedTelegramQrLogin() {
    try {
      const response = await telegramQrLogin.mutateAsync(wizard.buildTelegramQrLoginRequest())
      wizard.setSuccess(telegramQrResult(response))
    } catch (error) {
      wizard.setError(error instanceof Error ? error.message : t('Failed to start Telegram QR login.'))
    }
  }

  async function handleRefreshGuidedResult() {
    const setupId = wizard.guidedResult?.setupId
    if (!setupId) return

    try {
      const response = await telegramQrStatus.mutateAsync(setupId)
      wizard.setSuccess(telegramQrResult(response))
    } catch (error) {
      wizard.setError(error instanceof Error ? error.message : t('Failed to refresh Telegram QR login status.'))
    }
  }

  async function handleSubmit() {
    switch (selectedProvider.value.id) {
      case 'mail':
        await startManagedMailConnection()
        return
      case 'telegram':
        await startManagedTelegramQrLogin()
        return
      case 'whatsapp':
        await openManagedWhatsappCompanion()
        return
      case 'zoom':
        wizard.setBlocked({
          title: t('Workspace callback required'),
          message: t(
            'Zoom callback setup is available only after the workspace app credentials are provisioned.'
          ),
          blockers: [
            t('Settings stays callback-first and never falls back to raw client-id or secret fields.'),
            t('Callback and QR stay primary. Manual credentials stay hidden unless an exception recovery flow is explicitly required.'),
            t('Resume after the managed Zoom app is prepared for this workspace.'),
          ],
        })
        return
      case 'yandex_telemost':
        wizard.setBlocked({
          title: t('Workspace callback required'),
          message: t(
            'Yandex Telemost setup continues through a prepared callback route owned by the runtime.'
          ),
          blockers: [
            t('Settings does not expose manual OAuth or secret fields for Telemost.'),
            t('Resume after the workspace callback route is ready.'),
          ],
        })
        return
      default:
        wizard.setBlocked({
          title: t('Exception route required'),
          message: t(selectedProvider.value.summary),
          blockers: [
            t(selectedProvider.value.guidance),
            t(
              'Callback and QR stay primary. Manual credentials stay hidden unless an exception recovery flow is explicitly required.'
            ),
          ],
        })
    }
  }

  function closeWizard() {
    wizard.close()
  }

  return {
    activeFlow,
    activeFlowIcon,
    activeFlowId,
    canSubmit,
    canRefreshGuidedResult,
    closeWizard,
    exceptionFlowCard,
    guidedQrImage,
    handleRefreshGuidedResult,
    handleSubmit,
    isSubmitting,
    launchSteps,
    primaryFlowCards,
    providerStatus,
    selectedAccountLabel,
    selectedProvider,
    setActiveFlow,
    submitLabel,
    t,
    wizard,
  }
}

function telegramQrResult(response: TelegramQrLoginStatusResponse) {
  return {
    title: 'Telegram QR login started',
    message: response.message || `Status: ${response.status}`,
    setupId: response.setup_id,
    status: response.status,
    qrSvg: response.qr_svg ?? undefined,
    qrLink: response.qr_link ?? undefined,
  }
}
