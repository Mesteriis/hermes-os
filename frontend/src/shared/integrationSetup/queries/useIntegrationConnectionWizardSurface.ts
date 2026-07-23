import { computed, onScopeDispose, ref, toValue, watch, type MaybeRefOrGetter } from 'vue'
import { useMutation } from '@tanstack/vue-query'
import {
  useSetupImapEmailAccountMutation,
  useStartGmailOAuthSetupMutation,
} from '../../../integrations/mail/queries/accountSetupQueries'
import { useOpenWhatsappWebCompanionForPairingMutation } from '../../../integrations/whatsapp/queries/useWhatsappRuntimeQuery'
import { useI18n } from '../../../platform/i18n'
import {
  fetchTelegramQrLoginStatus,
  startTelegramQrLogin,
  submitTelegramQrLoginPassword,
  type TelegramQrLoginStatusResponse,
} from '../api/telegramQrLogin'
import {
  connectionProviderIdFromAccountKind,
  useIntegrationConnectionWizardStore,
} from '../../stores/integrationConnectionWizard'
import {
  shouldPollTelegramQrStatus,
  telegramQrDisplayMessage,
  telegramQrIsReady,
  telegramQrNeedsPassword,
  telegramQrResult,
} from './telegramQrWizardState'
import {
  FLOW_CATALOG,
  FLOW_ICONS,
  type ConnectionFlowCard,
} from './connectionFlowCatalog'
import type {
  ConnectionFlowPattern,
  ConnectionProviderId,
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

export interface ConnectionWizardCheck {
  label: string
  description: string
  status: 'ready' | 'pending' | 'blocked'
}

export interface ConnectionWizardCapability {
  label: string
  description: string
  enabled: boolean
  locked?: boolean
}
export type TelegramLoginMode = 'qr' | 'phone'

export function useIntegrationConnectionWizardSurface(options: {
  selectedAccount: MaybeRefOrGetter<SelectedIntegrationAccount | null | undefined>
  defaultProviderId: MaybeRefOrGetter<ConnectionProviderId | null | undefined>
}) {
  const { t } = useI18n()
  const wizard = useIntegrationConnectionWizardStore()
  const gmailOAuthSetup = useStartGmailOAuthSetupMutation()
  const imapAccountSetup = useSetupImapEmailAccountMutation()
  const telegramQrLogin = useMutation({ mutationFn: startTelegramQrLogin })
  const telegramQrStatus = useMutation({ mutationFn: fetchTelegramQrLoginStatus })
  const telegramQrPassword = useMutation({
    mutationFn: ({ setupId, password }: { setupId: string; password: string }) =>
      submitTelegramQrLoginPassword(setupId, { password }),
  })
  const hiddenWebviewMutation = useOpenWhatsappWebCompanionForPairingMutation()

  const activeFlowId = ref<ConnectionFlowPattern>('browser_callback')
  const gmailAccountLabel = ref('Personal Google')
  const icloudEmail = ref('')
  const icloudPassword = ref('')
  const icloudAccountLabel = ref('Personal iCloud')
  const telegramLoginMode = ref<TelegramLoginMode>('qr')
  const telegramPhone = ref('')
  const telegramCloudPassword = ref('')
  const telegramAccountLabel = ref('Personal Telegram')
  const telegramPasswordSubmitted = ref(false)
  const whatsappDeviceName = ref('Hermes Desktop')
  const whatsappAccountLabel = ref('Personal WhatsApp')
  let telegramStatusTimer: ReturnType<typeof setTimeout> | null = null

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
      resetProviderDraft(provider.id)
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
  const providerCards = computed(() =>
    wizard.providerCatalog.filter((provider) =>
      provider.id === 'mail' ||
      provider.id === 'icloud' ||
      provider.id === 'telegram'
    )
  )
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
            title: 'Start hidden companion',
            detail: 'The runtime remains hidden and never embeds pairing secrets here.',
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
    () =>
      gmailOAuthSetup.isPending.value ||
      imapAccountSetup.isPending.value ||
      telegramQrLogin.isPending.value ||
      telegramQrPassword.isPending.value ||
      hiddenWebviewMutation.isPending.value
  )
  const canSubmit = computed(() => {
    switch (selectedProvider.value.id) {
      case 'mail':
        return !gmailOAuthSetup.isPending.value
      case 'icloud':
        return Boolean(icloudEmail.value.trim()) &&
          Boolean(icloudPassword.value.trim()) &&
          !imapAccountSetup.isPending.value
      case 'telegram':
        return telegramLoginMode.value === 'qr'
          ? !telegramQrLogin.isPending.value
          : Boolean(telegramPhone.value.trim()) && !telegramQrLogin.isPending.value
      case 'whatsapp':
        return isSelectedWhatsAppAccount.value && !hiddenWebviewMutation.isPending.value
      default:
        return true
    }
  })
  const submitLabel = computed(() => {
    if (selectedProvider.value.id === 'mail' && gmailOAuthSetup.isPending.value) {
      return t('Opening browser callback...')
    }
    if (selectedProvider.value.id === 'icloud' && imapAccountSetup.isPending.value) {
      return t('Connecting iCloud...')
    }
    if (selectedProvider.value.id === 'whatsapp' && hiddenWebviewMutation.isPending.value) {
      return t('Opening QR companion...')
    }
    if (selectedProvider.value.id === 'telegram' && telegramQrLogin.isPending.value) {
      return t('Готовим QR...')
    }
    if (selectedProvider.value.id === 'telegram' && telegramLoginMode.value === 'qr') {
      return t('Показать QR')
    }
    if (selectedProvider.value.id === 'telegram' && telegramLoginMode.value === 'phone') {
      return t('Продолжить')
    }
    return t(selectedProvider.value.ctaLabel)
  })
  const selectedProviderChecks = computed<ConnectionWizardCheck[]>(() => {
    switch (selectedProvider.value.id) {
      case 'mail':
        return [
          { label: t('Аккаунт Google'), description: gmailOAuthSetup.isPending.value ? t('Открывается') : t('Готово'), status: gmailOAuthSetup.isPending.value ? 'pending' : 'ready' },
          { label: t('Почта'), description: wizard.guidedResult?.kind === 'success' ? t('Открыта') : t('Ожидает входа'), status: wizard.guidedResult?.kind === 'success' ? 'ready' : 'pending' },
          { label: t('Контакты и встречи'), description: wizard.guidedResult?.kind === 'success' ? t('Готово') : t('После входа'), status: wizard.guidedResult?.kind === 'success' ? 'ready' : 'pending' },
        ]
      case 'icloud':
        return [
          { label: t('Почта'), description: imapAccountSetup.isPending.value ? t('Проверяется') : statusDescription(), status: resultStatus() },
          { label: t('Папки'), description: wizard.guidedResult?.kind === 'success' ? t('Готово') : t('После подключения'), status: wizard.guidedResult?.kind === 'success' ? 'ready' : 'pending' },
          { label: t('Отправка'), description: wizard.guidedResult?.kind === 'success' ? t('Готово') : t('После подключения'), status: wizard.guidedResult?.kind === 'success' ? 'ready' : 'pending' },
        ]
      case 'telegram':
        return [
          { label: t('Вход'), description: telegramQrLogin.isPending.value ? t('Открывается') : statusDescription(), status: resultStatus() },
          { label: t('Чаты'), description: wizard.guidedResult?.kind === 'success' ? t('Проверяется') : t('После входа'), status: wizard.guidedResult?.kind === 'success' ? 'pending' : 'pending' },
          { label: t('Медиа'), description: t('После входа'), status: 'pending' },
        ]
      case 'whatsapp':
        return [
          { label: t('Устройство'), description: hiddenWebviewMutation.isPending.value ? t('Открывается') : statusDescription(), status: resultStatus() },
          { label: t('Чаты'), description: wizard.guidedResult?.kind === 'success' ? t('Проверяется') : t('После входа'), status: wizard.guidedResult?.kind === 'success' ? 'pending' : 'pending' },
          { label: t('Отправка'), description: t('После входа'), status: 'pending' },
        ]
      default:
        return []
    }
  })
  const selectedProviderCapabilities = computed<ConnectionWizardCapability[]>(() => {
    switch (selectedProvider.value.id) {
      case 'mail':
        return [
          { label: t('Почта'), description: t('Письма и папки'), enabled: true },
          { label: t('Контакты'), description: t('Люди и адреса'), enabled: true },
          { label: t('Google Drive'), description: t('Файлы и документы'), enabled: true },
          { label: t('Google Photos'), description: t('Фото и альбомы'), enabled: true },
          { label: t('Google Keep'), description: t('Заметки и списки'), enabled: true },
          { label: t('Google Meet'), description: t('Встречи и ссылки'), enabled: true },
        ]
      case 'icloud':
        return [
          { label: t('Почта'), description: t('Письма и папки'), enabled: true },
          { label: t('Отправка'), description: t('Исходящие письма'), enabled: true },
          { label: t('Контакты'), description: t('Адреса и профили'), enabled: false, locked: true },
        ]
      case 'telegram':
        return [
          { label: t('Сообщения'), description: t('Диалоги и группы'), enabled: true },
          { label: t('Контакты'), description: t('Люди и профили'), enabled: true },
          { label: t('Медиа'), description: t('Вложения и голосовые'), enabled: false },
        ]
      case 'whatsapp':
        return [
          { label: t('Чаты и группы'), description: t('Сообщения и участники'), enabled: true },
          { label: t('Медиа'), description: t('Фото, документы и голосовые'), enabled: false },
          { label: t('Статусы'), description: t('Обновления статусов'), enabled: false, locked: true },
        ]
      default:
        return []
    }
  })
  const guidedQrImage = computed(() => {
    const qrSvg = wizard.guidedResult?.qrSvg
    if (!qrSvg) return null
    return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(qrSvg)}`
  })
  const telegramQrStatusValue = computed(() =>
    selectedProvider.value.id === 'telegram' ? wizard.guidedResult?.status ?? null : null
  )
  const telegramQrPasswordRequired = computed(() => telegramQrNeedsPassword(telegramQrStatusValue.value))
  const telegramQrReady = computed(() => telegramQrIsReady(telegramQrStatusValue.value))
  const guidedResultMessage = computed(() => {
    if (selectedProvider.value.id !== 'telegram') return wizard.guidedResult?.message ?? ''
    return telegramQrDisplayMessage(telegramQrStatusValue.value, wizard.guidedResult?.message, t)
  })
  const canSubmitTelegramCloudPassword = computed(() =>
    telegramQrPasswordRequired.value &&
    Boolean(telegramCloudPassword.value.trim()) &&
    !telegramQrPassword.isPending.value
  )
  const showSelectedProviderChecks = computed(() =>
    selectedProvider.value.id !== 'telegram' || telegramQrReady.value
  )
  const canRefreshGuidedResult = computed(() =>
    selectedProvider.value.id === 'telegram' &&
    telegramQrReady.value &&
    Boolean(wizard.guidedResult?.setupId) &&
    !telegramQrStatus.isPending.value
  )

  watch(
    () => [
      selectedProvider.value.id,
      wizard.guidedResult?.setupId ?? '',
      wizard.guidedResult?.status ?? '',
      telegramQrStatus.isPending.value,
      telegramQrLogin.isPending.value,
      telegramQrPassword.isPending.value,
      telegramPasswordSubmitted.value,
    ] as const,
    scheduleTelegramStatusRefresh
  )

  onScopeDispose(clearTelegramStatusTimer)

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

  function previewProvider(providerId: ConnectionProviderId) {
    wizard.previewProvider(providerId)
    activeFlowId.value = selectedProvider.value.flowPattern
    resetProviderDraft(providerId)
  }

  async function startManagedMailConnection() {
    try {
      const selectedAccount = toValue(options.selectedAccount) ?? null
      const displayName = selectedAccountLabel.value ?? gmailAccountLabel.value
      const response = await gmailOAuthSetup.mutateAsync(
        wizard.buildGmailOAuthRequest(displayName, selectedAccount)
      )
      if (typeof window !== 'undefined' && typeof window.open === 'function') {
        window.open(response.authorization_url, '_blank', 'noopener,noreferrer')
      }
      wizard.setSuccess({
        title: t('Вход открыт'),
        message: t('Завершите вход в браузере.'),
      })
    } catch (error) {
      wizard.setError(error instanceof Error ? error.message : t('Не удалось открыть вход Google.'))
    }
  }

  async function setupIcloudConnection() {
    const email = icloudEmail.value.trim()
    const password = icloudPassword.value.trim()
    if (!email || !password) {
      wizard.setError(t('Укажите email и пароль приложения.'))
      return
    }

    try {
      await imapAccountSetup.mutateAsync({
        account_id: makeIcloudAccountId(email),
        provider_kind: 'icloud',
        display_name: icloudAccountLabel.value.trim() || email,
        external_account_id: email,
        host: 'imap.mail.me.com',
        port: 993,
        tls: true,
        mailbox: 'INBOX',
        username: email,
        password,
        secret_kind: 'app_password',
        smtp_host: 'smtp.mail.me.com',
        smtp_port: 587,
        smtp_tls: false,
        smtp_starttls: true,
        smtp_username: email,
      })
      wizard.setSuccess({
        title: t('iCloud подключён'),
        message: t('Почта iCloud готова.'),
      })
      icloudPassword.value = ''
    } catch (error) {
      wizard.setError(error instanceof Error ? error.message : t('Не удалось подключить iCloud.'))
    }
  }

  async function openManagedWhatsappCompanion() {
    if (!selectedAccountId.value || !isSelectedWhatsAppAccount.value) {
      wizard.setBlocked({
        title: t('Выберите WhatsApp аккаунт'),
        message: t('Откройте мастер из выбранного аккаунта WhatsApp.'),
      })
      return
    }

    try {
      const manifest = await hiddenWebviewMutation.mutateAsync({ account_id: selectedAccountId.value })
      wizard.setSuccess({
        title: t('WhatsApp WebView открыт для pairing'),
        message: manifest.reused_existing_window
          ? t('WhatsApp WebView уже был открыт.')
          : t('Отсканируйте QR-код, затем WebView можно скрыть.'),
      })
    } catch (error) {
      wizard.setError(
        error instanceof Error ? error.message : t('Не удалось открыть WhatsApp.')
      )
    }
  }

  async function startManagedTelegramQrLogin() {
    try {
      telegramPasswordSubmitted.value = false
      const response = await telegramQrLogin.mutateAsync(
        wizard.buildTelegramQrLoginRequest(telegramAccountLabel.value)
      )
      setTelegramQrResult(response)
    } catch (error) {
      wizard.setError(telegramQrErrorMessage(error))
    }
  }

  function startManagedTelegramPhoneLogin() {
    wizard.setBlocked({
      title: t('Телефонный вход'),
      message: t('Пока доступен QR-вход.'),
    })
  }

  async function handleRefreshGuidedResult() {
    const setupId = wizard.guidedResult?.setupId
    if (!setupId) return

    try {
      const response = await telegramQrStatus.mutateAsync(setupId)
      setTelegramQrResult(response)
    } catch (error) {
      wizard.setError(telegramQrErrorMessage(error))
    }
  }

  async function ensureTelegramQrLoginStarted() {
    if (selectedProvider.value.id !== 'telegram' || telegramLoginMode.value !== 'qr') return
    if (telegramQrLogin.isPending.value || wizard.guidedResult?.setupId) return
    await startManagedTelegramQrLogin()
  }

  async function submitTelegramCloudPassword() {
    const setupId = wizard.guidedResult?.setupId
    const password = telegramCloudPassword.value.trim()
    if (!setupId || !password || !telegramQrPasswordRequired.value) return

    try {
      telegramPasswordSubmitted.value = true
      const response = await telegramQrPassword.mutateAsync({ setupId, password })
      setTelegramQrResult(response)
    } catch (error) {
      wizard.setError(telegramQrErrorMessage(error))
    } finally {
      telegramCloudPassword.value = ''
    }
  }

  async function handleSubmit() {
    switch (selectedProvider.value.id) {
      case 'mail':
        await startManagedMailConnection()
        return
      case 'icloud':
        await setupIcloudConnection()
        return
      case 'telegram':
        if (telegramLoginMode.value === 'phone') {
          startManagedTelegramPhoneLogin()
          return
        }
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

  function resetProviderDraft(providerId: ConnectionProviderId) {
    if (providerId === 'mail' && !gmailAccountLabel.value.trim()) {
      gmailAccountLabel.value = 'Personal Google'
    }
    if (providerId === 'icloud') {
      icloudPassword.value = ''
      if (!icloudAccountLabel.value.trim()) icloudAccountLabel.value = 'Personal iCloud'
    }
    if (providerId === 'telegram' && !telegramAccountLabel.value.trim()) {
      telegramAccountLabel.value = 'Personal Telegram'
    }
    if (providerId === 'telegram') {
      telegramLoginMode.value = 'qr'
    }
    if (providerId === 'whatsapp' && !whatsappAccountLabel.value.trim()) {
      whatsappAccountLabel.value = 'Personal WhatsApp'
    }
  }

  function resultStatus(): ConnectionWizardCheck['status'] {
    if (wizard.guidedResult?.kind === 'success') return 'ready'
    if (wizard.guidedResult?.kind === 'blocked' || wizard.guidedResult?.kind === 'error') return 'blocked'
    return 'pending'
  }

  function statusDescription(): string {
    if (wizard.guidedResult?.kind === 'success') return t('Готово')
    if (wizard.guidedResult?.kind === 'blocked') return t('Нужно действие')
    if (wizard.guidedResult?.kind === 'error') return t('Ошибка')
    return t('Ожидает')
  }

  function setTelegramLoginMode(mode: TelegramLoginMode) {
    telegramLoginMode.value = mode
  }

  function setTelegramQrResult(response: TelegramQrLoginStatusResponse) {
    wizard.setSuccess(telegramQrResult(response))
  }

  function scheduleTelegramStatusRefresh() {
    clearTelegramStatusTimer()
    if (selectedProvider.value.id !== 'telegram') return
    const setupId = wizard.guidedResult?.setupId
    const status = wizard.guidedResult?.status
    if (!setupId || telegramQrStatus.isPending.value || telegramQrLogin.isPending.value || telegramQrPassword.isPending.value) {
      return
    }
    if (status === 'waiting_password' && /rejected|try again/i.test(wizard.guidedResult?.message ?? '')) {
      telegramPasswordSubmitted.value = false
      return
    }
    if (!shouldPollTelegramQrStatus(status, telegramPasswordSubmitted.value)) return
    telegramStatusTimer = setTimeout(() => {
      void handleRefreshGuidedResult()
    }, 1_500)
  }

  function clearTelegramStatusTimer() {
    if (!telegramStatusTimer) return
    clearTimeout(telegramStatusTimer)
    telegramStatusTimer = null
  }

  function telegramQrErrorMessage(error: unknown): string {
    const message = errorMessage(error)
    if (message.includes('api_id must not be empty') || message.includes('api_hash must not be empty')) {
      return t('Не настроены Telegram API ID/API Hash.')
    }
    return message || t('Не удалось открыть Telegram QR.')
  }

  function errorMessage(error: unknown): string {
    if (error instanceof Error) return error.message
    if (typeof error === 'object' && error !== null) {
      const value = (error as { message?: unknown }).message
      if (typeof value === 'string') return value
    }
    return ''
  }

  function makeIcloudAccountId(email: string): string {
    const stableEmail = email
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '') || 'account'
    return `icloud-${stableEmail}`
  }

  return {
    activeFlow,
    activeFlowIcon,
    activeFlowId,
    canSubmit,
    canSubmitTelegramCloudPassword,
    canRefreshGuidedResult,
    closeWizard,
    ensureTelegramQrLoginStarted,
    exceptionFlowCard,
    guidedQrImage,
    guidedResultMessage,
    gmailAccountLabel,
    handleRefreshGuidedResult,
    handleSubmit,
    icloudAccountLabel,
    icloudEmail,
    icloudPassword,
    isSubmitting,
    launchSteps,
    previewProvider,
    primaryFlowCards,
    providerCards,
    providerStatus,
    selectedProviderCapabilities,
    selectedProviderChecks,
    selectedAccountLabel,
    selectedProvider,
    setActiveFlow,
    setTelegramLoginMode,
    showSelectedProviderChecks,
    submitTelegramCloudPassword,
    submitLabel,
    telegramAccountLabel,
    telegramCloudPassword,
    telegramLoginMode,
    telegramQrNeedsPassword: telegramQrPasswordRequired,
    telegramPhone,
    t,
    whatsappAccountLabel,
    whatsappDeviceName,
    wizard,
  }
}
