import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { ConnectionProviderId } from '../../../shared/stores/integrationConnectionWizard'
import {
  useDeleteMailAccountMutation,
  useExportMailAccountSettingsMutation,
  useLogoutMailAccountMutation,
  useProviderAccountsQuery,
} from './useSettingsQuery'
import { useSettingsStore } from '../stores/settings'
import type { ProviderAccount } from '../types/settings'

type ConnectionModeId = 'browser_callback' | 'qr_companion'

export interface IntegrationAccountRow {
  account: ProviderAccount
  displayName: string
  icon: string
  providerLabel: string
  flowLabel: string
  statusText: string
  statusClass: string
  isSelected: boolean
}

export interface IntegrationGroup {
  label: string
  items: IntegrationAccountRow[]
}

export interface ConnectionModeProvider {
  id: ConnectionProviderId
  icon: string
  label: string
}

export interface ConnectionModeCard {
  id: ConnectionModeId
  icon: string
  label: string
  description: string
  note: string
  defaultProviderId: ConnectionProviderId
  providers: ConnectionModeProvider[]
  configuredCount: number
}

export interface SelectedIntegrationSummary {
  account: ProviderAccount
  displayName: string
  providerLabel: string
  flowLabel: string
  email: string
  statusText: string
  nextStepLabel: string
  selectedInspectorActionLabel: string
  canManageMail: boolean
}

const PROVIDER_ICONS: Record<string, string> = {
  gmail: 'tabler:mail',
  icloud: 'tabler:cloud',
  imap: 'tabler:server',
  telegram_user: 'tabler:brand-telegram',
  telegram_bot: 'tabler:robot',
  whatsapp_web: 'tabler:brand-whatsapp',
  whatsapp_business_cloud: 'tabler:brand-whatsapp',
  zoom_user: 'tabler:video',
  zoom_server_to_server: 'tabler:video-plus',
  yandex_telemost_user: 'tabler:video-plus',
  zulip_bot: 'tabler:message-bolt',
}

const PROVIDER_LABELS: Record<string, string> = {
  gmail: 'Gmail',
  icloud: 'iCloud',
  imap: 'IMAP',
  telegram_user: 'Telegram User',
  telegram_bot: 'Telegram Bot',
  whatsapp_web: 'WhatsApp Web',
  whatsapp_business_cloud: 'WhatsApp Business Cloud',
  zoom_user: 'Zoom (OAuth/Live)',
  zoom_server_to_server: 'Zoom (Server-to-Server)',
  yandex_telemost_user: 'Yandex Telemost',
  zulip_bot: 'Zulip Bot',
}

function isMailProvider(providerKind: string): boolean {
  return providerKind === 'gmail' || providerKind === 'icloud' || providerKind === 'imap'
}

function isZoomProvider(providerKind: string): boolean {
  return providerKind === 'zoom_user' || providerKind === 'zoom_server_to_server'
}

function isTelegramProvider(providerKind: string): boolean {
  return providerKind === 'telegram_user' || providerKind === 'telegram_bot'
}

function isWhatsappProvider(providerKind: string): boolean {
  return providerKind === 'whatsapp_web' || providerKind === 'whatsapp_business_cloud'
}

function isYandexTelemostProvider(providerKind: string): boolean {
  return providerKind === 'yandex_telemost_user'
}

function isZulipProvider(providerKind: string): boolean {
  return providerKind === 'zulip_bot'
}

function isExceptionOnlyProvider(providerKind: string): boolean {
  return providerKind === 'icloud' || providerKind === 'imap'
}

function isExceptionRouteProvider(providerKind: string): boolean {
  return isTelegramProvider(providerKind) || isZulipProvider(providerKind)
}

function isManagedRuntimeProvider(providerKind: string): boolean {
  return (
    isZoomProvider(providerKind) ||
    isTelegramProvider(providerKind) ||
    isWhatsappProvider(providerKind) ||
    isYandexTelemostProvider(providerKind) ||
    isZulipProvider(providerKind)
  )
}

function providerIcon(providerKind: string): string {
  return PROVIDER_ICONS[providerKind] ?? 'tabler:plug-connected'
}

function providerLabel(providerKind: string): string {
  return PROVIDER_LABELS[providerKind] ?? providerKind
}

function providerDisplayName(account: ProviderAccount): string {
  return (
    account.display_name ||
    account.label ||
    account.email ||
    (typeof account.config?.email === 'string' ? account.config.email : null) ||
    account.external_account_id ||
    account.account_id
  )
}

function selectedAccountEmail(account: ProviderAccount): string {
  return account.email || (typeof account.config?.email === 'string' ? account.config.email : '')
}

function defaultProviderIdFromAccount(
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

function downloadJson(filename: string, value: unknown) {
  const blob = new Blob([JSON.stringify(value, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()
  link.remove()
  URL.revokeObjectURL(url)
}

export function useIntegrationsSettingsSurface() {
  const { t } = useI18n()
  const store = useSettingsStore()
  const { data: accountsData } = useProviderAccountsQuery()
  const exportMailMutation = useExportMailAccountSettingsMutation()
  const logoutMailMutation = useLogoutMailAccountMutation()
  const deleteMailMutation = useDeleteMailAccountMutation()

  const isConnectWizardOpen = ref(false)
  const activeMailAction = ref<string | null>(null)
  const connectWizardProviderId = ref<ConnectionProviderId | null>('mail')

  const accounts = computed(() => accountsData.value?.items ?? [])
  const selectedAccount = computed(() => {
    if (!store.selectedIntegrationId) return null
    return accounts.value.find((account) => account.account_id === store.selectedIntegrationId) ?? null
  })

  const providerFlowLabel = (providerKind: string): string => {
    if (providerKind === 'gmail') return t('Browser callback')
    if (isExceptionOnlyProvider(providerKind)) return t('Exception-only recovery')
    if (isWhatsappProvider(providerKind)) return t('QR companion')
    if (isExceptionRouteProvider(providerKind)) return t('Exception route')
    if (isZoomProvider(providerKind) || isYandexTelemostProvider(providerKind)) {
      return t('Browser callback')
    }
    return t('Managed flow')
  }

  const nextStepLabel = (providerKind: string): string => {
    if (providerKind === 'gmail') return t('Resume in browser callback flow')
    if (isExceptionOnlyProvider(providerKind)) {
      return t('Handle only through explicit exception recovery')
    }
    if (isWhatsappProvider(providerKind)) return t('Resume in visible QR companion')
    if (isExceptionRouteProvider(providerKind)) {
      return t('Continue in dedicated runtime')
    }
    if (isZoomProvider(providerKind) || isYandexTelemostProvider(providerKind)) {
      return t('Resume through the workspace callback route')
    }
    return t('Use managed setup flow')
  }

  const statusText = (account: ProviderAccount): string => {
    if (typeof account.is_authenticated === 'boolean' && !account.is_authenticated) {
      return t('Not authenticated')
    }
    if (typeof account.is_active === 'boolean' && !account.is_active) return t('Inactive')
    if (isManagedRuntimeProvider(account.provider_kind)) return t('Configured')
    return t('Active')
  }

  const statusClass = (account: ProviderAccount): string => {
    if (typeof account.is_authenticated === 'boolean' && !account.is_authenticated) {
      return 'unauthenticated'
    }
    if (typeof account.is_active === 'boolean' && !account.is_active) return 'inactive'
    if (isManagedRuntimeProvider(account.provider_kind)) return 'configured'
    return 'active'
  }

  const selectedInspectorActionLabel = (account: ProviderAccount | null): string => {
    if (!account) return t('Open connection wizard')
    if (isWhatsappProvider(account.provider_kind)) return t('Resume QR companion')
    if (account.provider_kind === 'gmail') return t('Resume browser callback')
    if (isExceptionRouteProvider(account.provider_kind)) return t('View exception route')
    if (isExceptionOnlyProvider(account.provider_kind)) return t('Review exception posture')
    return t('Review managed route')
  }

  const toIntegrationAccountRow = (account: ProviderAccount): IntegrationAccountRow => ({
    account,
    displayName: providerDisplayName(account),
    icon: providerIcon(account.provider_kind),
    providerLabel: providerLabel(account.provider_kind),
    flowLabel: providerFlowLabel(account.provider_kind),
    statusText: statusText(account),
    statusClass: statusClass(account),
    isSelected: store.selectedIntegrationId === account.account_id,
  })

  const groups = computed<IntegrationGroup[]>(() => {
    const rows = [
      {
        label: t('Mail accounts'),
        items: accounts.value.filter((account) => isMailProvider(account.provider_kind)),
      },
      {
        label: t('Telegram accounts'),
        items: accounts.value.filter((account) => isTelegramProvider(account.provider_kind)),
      },
      {
        label: t('WhatsApp accounts'),
        items: accounts.value.filter((account) => isWhatsappProvider(account.provider_kind)),
      },
      {
        label: t('Zoom accounts'),
        items: accounts.value.filter((account) => isZoomProvider(account.provider_kind)),
      },
      {
        label: t('Yandex Telemost accounts'),
        items: accounts.value.filter((account) => isYandexTelemostProvider(account.provider_kind)),
      },
      {
        label: t('Zulip accounts'),
        items: accounts.value.filter((account) => isZulipProvider(account.provider_kind)),
      },
      {
        label: t('Other accounts'),
        items: accounts.value.filter((account) =>
          !isMailProvider(account.provider_kind) &&
          !isTelegramProvider(account.provider_kind) &&
          !isWhatsappProvider(account.provider_kind) &&
          !isZoomProvider(account.provider_kind) &&
          !isYandexTelemostProvider(account.provider_kind) &&
          !isZulipProvider(account.provider_kind)
        ),
      },
    ]

    const nonEmptyGroups = rows
      .filter((group) => group.items.length > 0)
      .map((group) => ({ label: group.label, items: group.items.map(toIntegrationAccountRow) }))

    if (nonEmptyGroups.length > 0) return nonEmptyGroups
    return [{ label: t('Accounts'), items: accounts.value.map(toIntegrationAccountRow) }]
  })
  const hasAccounts = computed(() => accounts.value.length > 0)

  const connectionModes = computed<ConnectionModeCard[]>(() => [
    {
      id: 'browser_callback',
      icon: 'tabler:browser-share',
      label: 'Browser callbacks',
      description:
        'Launch provider authorization in a secure browser flow and let Hermes resume on callback return.',
      note: 'Primary path for managed OAuth providers.',
      defaultProviderId: 'mail',
      providers: [
        { id: 'mail', icon: 'tabler:mail', label: 'Gmail' },
        { id: 'zoom', icon: 'tabler:video', label: 'Zoom' },
        { id: 'yandex_telemost', icon: 'tabler:video-plus', label: 'Yandex Telemost' },
      ],
      configuredCount: accounts.value.filter((account) =>
        account.provider_kind === 'gmail' ||
        isZoomProvider(account.provider_kind) ||
        isYandexTelemostProvider(account.provider_kind)
      ).length,
    },
    {
      id: 'qr_companion',
      icon: 'tabler:qrcode',
      label: 'QR companion',
      description:
        'Open a visible runtime companion, scan the QR code from your phone and finish linking outside Settings.',
      note: 'Used for device pairing without exposing session material.',
      defaultProviderId: 'whatsapp',
      providers: [
        { id: 'whatsapp', icon: 'tabler:brand-whatsapp', label: 'WhatsApp Web' },
      ],
      configuredCount: accounts.value.filter((account) => isWhatsappProvider(account.provider_kind)).length,
    },
  ])

  const selectedAccountSummary = computed<SelectedIntegrationSummary | null>(() => {
    if (!selectedAccount.value) return null
    return {
      account: selectedAccount.value,
      displayName: providerDisplayName(selectedAccount.value),
      providerLabel: providerLabel(selectedAccount.value.provider_kind),
      flowLabel: providerFlowLabel(selectedAccount.value.provider_kind),
      email: selectedAccountEmail(selectedAccount.value),
      statusText: statusText(selectedAccount.value),
      nextStepLabel: nextStepLabel(selectedAccount.value.provider_kind),
      selectedInspectorActionLabel: selectedInspectorActionLabel(selectedAccount.value),
      canManageMail: isMailProvider(selectedAccount.value.provider_kind),
    }
  })

  function openConnectWizard(providerId: ConnectionProviderId | null = null) {
    connectWizardProviderId.value = providerId ?? defaultProviderIdFromAccount(selectedAccount.value?.provider_kind)
    isConnectWizardOpen.value = true
  }

  function closeConnectWizard() {
    isConnectWizardOpen.value = false
    connectWizardProviderId.value = 'mail'
  }

  function selectIntegration(accountId: string | null) {
    store.selectIntegration(accountId)
  }

  async function handleExport(accountId: string) {
    activeMailAction.value = accountId
    try {
      const result = await exportMailMutation.mutateAsync(accountId)
      if (result.result) {
        downloadJson(`mail-account-${accountId}-${result.result.exported_at}.json`, result.result)
        store.setActionMessage(t('Mail account export snapshot prepared'))
      }
    } catch (error) {
      store.setError(error instanceof Error ? error.message : t('Export failed'))
    } finally {
      activeMailAction.value = null
    }
  }

  async function handleLogout(accountId: string) {
    activeMailAction.value = accountId
    try {
      await logoutMailMutation.mutateAsync(accountId)
      store.setActionMessage(t('Mail account logged out'))
    } catch (error) {
      store.setError(error instanceof Error ? error.message : t('Logout failed'))
    } finally {
      activeMailAction.value = null
    }
  }

  async function handleDelete(accountId: string) {
    activeMailAction.value = accountId
    try {
      await deleteMailMutation.mutateAsync(accountId)
      if (store.selectedIntegrationId === accountId) {
        store.selectIntegration(null)
      }
      store.setActionMessage(t('Mail account deleted'))
    } catch (error) {
      store.setError(error instanceof Error ? error.message : t('Delete failed'))
    } finally {
      activeMailAction.value = null
    }
  }

  return {
    actionMessage: computed(() => store.actionMessage),
    activeMailAction,
    closeConnectWizard,
    connectWizardProviderId,
    connectionModes,
    groups,
    isConnectWizardOpen,
    openConnectWizard,
    selectIntegration,
    selectedAccount,
    selectedAccountSummary,
    errorMessage: computed(() => store.errorMessage),
    hasAccounts,
    handleDelete,
    handleExport,
    handleLogout,
  }
}
