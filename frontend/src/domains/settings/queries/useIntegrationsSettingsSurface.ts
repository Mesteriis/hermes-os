import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import {
  useMailSyncSettingsQuery,
  useUpdateMailSyncSettingsMutation,
} from '../../../shared/mailSync/runtimeQueries'
import type { ConnectionProviderId } from '../../../shared/stores/integrationConnectionWizard'
import {
  useDeleteMailAccountMutation,
  useExportMailAccountSettingsMutation,
  useCalendarAccountsQuery,
  useLogoutMailAccountMutation,
  useProviderAccountsQuery,
  useUpdateCalendarAccountMutation,
  useUpdateProviderAccountMutation,
} from './useSettingsQuery'
import { useSettingsStore } from '../stores/settings'
import type { CalendarAccount, ProviderAccount } from '../types/settings'

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
  canOpenSetupAction: boolean
  accountEnabled: boolean
  accountToggleLabel: string
  accountToggleHelp: string
  labelDraft: string
  labelDirty: boolean
  labelSaving: boolean
  services: AccountServiceRow[]
}

export interface AccountServiceRow {
  id: 'mail' | 'calendar' | 'contacts' | 'messenger' | 'meetings'
  label: string
  icon: string
  enabled: boolean
  statusText: string
  detail: string
  canToggle: boolean
  disabledReason?: string
  isBusy: boolean
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

function supportsCalendar(providerKind: string): boolean {
  return providerKind === 'gmail' || providerKind === 'icloud'
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
  return isZulipProvider(providerKind)
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

function providerAccountEnabled(account: ProviderAccount): boolean {
  if (typeof account.is_authenticated === 'boolean' && !account.is_authenticated) return false
  if (typeof account.is_active === 'boolean') return account.is_active
  return account.config?.auth_state !== 'logged_out'
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
  const { data: calendarAccountsData } = useCalendarAccountsQuery()
  const exportMailMutation = useExportMailAccountSettingsMutation()
  const logoutMailMutation = useLogoutMailAccountMutation()
  const deleteMailMutation = useDeleteMailAccountMutation()
  const updateMailSyncSettingsMutation = useUpdateMailSyncSettingsMutation()
  const updateCalendarAccountMutation = useUpdateCalendarAccountMutation()
  const updateProviderAccountMutation = useUpdateProviderAccountMutation()

  const isConnectWizardOpen = ref(false)
  const activeMailAction = ref<string | null>(null)
  const selectedAccountLabelDraft = ref('')
  const connectWizardProviderId = ref<ConnectionProviderId | null>('mail')
  const connectWizardUsesSelectedAccount = ref(false)

  const accounts = computed(() => accountsData.value?.items ?? [])
  const calendarAccounts = computed(() => calendarAccountsData.value?.items ?? [])
  const selectedAccount = computed(() => {
    if (!store.selectedIntegrationId) return null
    return accounts.value.find((account) => account.account_id === store.selectedIntegrationId) ?? null
  })
  const selectedMailAccountId = computed(() => {
    const account = selectedAccount.value
    return account && isMailProvider(account.provider_kind) ? account.account_id : null
  })
  const selectedMailSyncSettingsQuery = useMailSyncSettingsQuery(() => selectedMailAccountId.value)
  const selectedMailSyncSettings = computed(() => selectedMailSyncSettingsQuery.data.value ?? null)
  const selectedCalendarAccount = computed(() => {
    const account = selectedAccount.value
    if (!account) return null
    return calendarAccounts.value.find((calendarAccount) => matchesCalendarAccount(account, calendarAccount)) ?? null
  })
  const selectedAccountLabelDirty = computed(() => {
    const account = selectedAccount.value
    if (!account) return false
    return selectedAccountLabelDraft.value.trim() !== account.display_name.trim()
  })

  watch(selectedAccount, (account) => {
    selectedAccountLabelDraft.value = account?.display_name ?? ''
  }, { immediate: true })

  const providerFlowLabel = (providerKind: string): string => {
    if (providerKind === 'gmail') return t('Browser callback')
    if (isTelegramProvider(providerKind)) return t('QR companion')
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
    if (isTelegramProvider(providerKind)) return t('Start or resume Telegram QR login')
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
    if (isTelegramProvider(account.provider_kind)) return t('Resume Telegram QR login')
    if (account.provider_kind === 'gmail') return t('Resume browser callback')
    if (isExceptionRouteProvider(account.provider_kind)) return t('View exception route')
    if (isExceptionOnlyProvider(account.provider_kind)) return t('No self-serve setup route')
    return t('Review managed route')
  }

  const canOpenSetupAction = (account: ProviderAccount): boolean => {
    return (
      account.provider_kind === 'gmail' ||
      isTelegramProvider(account.provider_kind) ||
      isWhatsappProvider(account.provider_kind) ||
      isZoomProvider(account.provider_kind) ||
      isYandexTelemostProvider(account.provider_kind) ||
      isExceptionRouteProvider(account.provider_kind)
    )
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

  const selectedAccountSummary = computed<SelectedIntegrationSummary | null>(() => {
    if (!selectedAccount.value) return null
    const account = selectedAccount.value
    const accountEnabled = providerAccountEnabled(account)
    return {
      account,
      displayName: providerDisplayName(account),
      providerLabel: providerLabel(account.provider_kind),
      flowLabel: providerFlowLabel(account.provider_kind),
      email: selectedAccountEmail(account),
      statusText: statusText(account),
      nextStepLabel: nextStepLabel(account.provider_kind),
      selectedInspectorActionLabel: selectedInspectorActionLabel(account),
      canManageMail: isMailProvider(account.provider_kind),
      canOpenSetupAction: canOpenSetupAction(account),
      accountEnabled,
      accountToggleLabel: accountEnabled ? t('Account enabled') : t('Account disabled'),
      accountToggleHelp: accountEnabled
        ? t('Disabling a mail account logs it out and stops mail sync through the backend.')
        : t('Enabling opens the managed setup wizard because credentials are not editable here.'),
      labelDraft: selectedAccountLabelDraft.value,
      labelDirty: selectedAccountLabelDirty.value,
      labelSaving: updateProviderAccountMutation.isPending.value,
      services: serviceRowsForAccount(account),
    }
  })

  function openConnectWizard(providerId: ConnectionProviderId | null = null) {
    connectWizardProviderId.value = providerId ?? defaultProviderIdFromAccount(selectedAccount.value?.provider_kind)
    connectWizardUsesSelectedAccount.value = providerId === null && Boolean(selectedAccount.value)
    isConnectWizardOpen.value = true
  }

  function closeConnectWizard() {
    isConnectWizardOpen.value = false
    connectWizardProviderId.value = 'mail'
    connectWizardUsesSelectedAccount.value = false
  }

  const connectWizardSelectedAccount = computed(() => connectWizardUsesSelectedAccount.value ? selectedAccount.value : null)

  function selectIntegration(accountId: string | null) {
    store.selectIntegration(accountId)
  }

  function handleSelectedAccountLabelInput(value: string) {
    selectedAccountLabelDraft.value = value
  }

  async function handleSaveSelectedAccountLabel() {
    const account = selectedAccount.value
    if (!account) return
    const displayName = selectedAccountLabelDraft.value.trim()
    if (!displayName) {
      store.setError(t('Label cannot be empty'))
      return
    }
    if (displayName === account.display_name.trim()) return

    store.clearMessages()
    try {
      await updateProviderAccountMutation.mutateAsync({
        accountId: account.account_id,
        update: { display_name: displayName },
      })
      store.setActionMessage(t('Account label saved'))
    } catch (error) {
      store.setError(error instanceof Error ? error.message : t('Account label update failed'))
    }
  }

  async function handleExport(accountId: string) {
    activeMailAction.value = accountId
    try {
      const result = await exportMailMutation.mutateAsync(accountId)
      downloadJson(`mail-account-${accountId}-${result.exported_at}.json`, result)
      store.setActionMessage(t('Mail account export snapshot prepared'))
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

  async function handleToggleSelectedAccount(enabled: boolean) {
    const account = selectedAccount.value
    if (!account) return

    if (!isMailProvider(account.provider_kind)) {
      store.setError(t('This provider does not expose a generic account toggle contract yet.'))
      return
    }

    if (enabled) {
      if (account.provider_kind !== 'gmail') {
        store.setError(t('This mail provider does not expose a self-serve reconnect flow in Settings yet.'))
        return
      }
      openConnectWizard(defaultProviderIdFromAccount(account.provider_kind))
      return
    }

    await handleLogout(account.account_id)
  }

  async function handleToggleSelectedService(serviceId: AccountServiceRow['id'], enabled: boolean) {
    const account = selectedAccount.value
    if (!account) return

    if (serviceId === 'mail') {
      await handleToggleMailService(account, enabled)
      return
    }

    if (serviceId === 'calendar') {
      await handleToggleCalendarService(enabled)
      return
    }

    store.setError(t('This service does not expose a settings toggle contract yet.'))
  }

  async function handleToggleMailService(account: ProviderAccount, enabled: boolean) {
    if (!isMailProvider(account.provider_kind)) return
    const current = selectedMailSyncSettings.value
    if (!current) {
      store.setError(t('Mail sync settings are not loaded yet.'))
      return
    }

    activeMailAction.value = account.account_id
    store.clearMessages()
    try {
      await updateMailSyncSettingsMutation.mutateAsync({
        accountId: account.account_id,
        settings: {
          sync_enabled: enabled,
          batch_size: current.batch_size,
          poll_interval_seconds: current.poll_interval_seconds,
        },
      })
      store.setActionMessage(enabled ? t('Mail service enabled') : t('Mail service disabled'))
    } catch (error) {
      store.setError(error instanceof Error ? error.message : t('Mail service update failed'))
    } finally {
      activeMailAction.value = null
    }
  }

  async function handleToggleCalendarService(enabled: boolean) {
    const calendarAccount = selectedCalendarAccount.value
    if (!calendarAccount) {
      store.setError(t('No matching calendar account contract is available for this provider account.'))
      return
    }

    activeMailAction.value = calendarAccount.account_id
    store.clearMessages()
    try {
      await updateCalendarAccountMutation.mutateAsync({
        accountId: calendarAccount.account_id,
        update: {
          sync_status: enabled ? 'active' : 'paused',
        },
      })
      store.setActionMessage(enabled ? t('Calendar service enabled') : t('Calendar service paused'))
    } catch (error) {
      store.setError(error instanceof Error ? error.message : t('Calendar service update failed'))
    } finally {
      activeMailAction.value = null
    }
  }

  function serviceRowsForAccount(account: ProviderAccount): AccountServiceRow[] {
    const rows: AccountServiceRow[] = []
    const mailSyncSettings = selectedMailSyncSettings.value
    const calendarAccount = selectedCalendarAccount.value

    if (isMailProvider(account.provider_kind)) {
      rows.push({
        id: 'mail',
        label: t('Mail'),
        icon: 'tabler:mail',
        enabled: mailSyncSettings?.sync_enabled ?? providerAccountEnabled(account),
        statusText: mailSyncSettings?.sync_enabled ? t('Sync enabled') : t('Sync paused'),
        detail: mailSyncSettings
          ? t('Uses backend mail sync settings for this account.')
          : t('Waiting for mail sync settings from the backend.'),
        canToggle: Boolean(mailSyncSettings),
        disabledReason: mailSyncSettings ? undefined : t('Sync settings are still loading.'),
        isBusy: updateMailSyncSettingsMutation.isPending.value && activeMailAction.value === account.account_id,
      })
    }

    if (supportsCalendar(account.provider_kind) || calendarAccount) {
      const enabled = Boolean(calendarAccount && calendarAccount.sync_status !== 'paused' && calendarAccount.sync_status !== 'disabled')
      rows.push({
        id: 'calendar',
        label: t('Calendar'),
        icon: 'tabler:calendar',
        enabled,
        statusText: calendarAccount ? calendarAccount.sync_status : t('Not configured'),
        detail: calendarAccount
          ? t('Uses the Calendar account sync_status contract.')
          : t('No matching Calendar account exists for this provider account.'),
        canToggle: Boolean(calendarAccount),
        disabledReason: calendarAccount ? undefined : t('Calendar account endpoint has no linked account yet.'),
        isBusy: updateCalendarAccountMutation.isPending.value,
      })
    }

    rows.push({
      id: 'contacts',
      label: t('Contacts'),
      icon: 'tabler:address-book',
      enabled: false,
      statusText: t('No contract'),
      detail: t('Contacts service toggle is hidden until the Contacts API contract exists.'),
      canToggle: false,
      disabledReason: t('Contacts account API is not present in the current backend contracts.'),
      isBusy: false,
    })

    if (isTelegramProvider(account.provider_kind) || isWhatsappProvider(account.provider_kind)) {
      rows.push({
        id: 'messenger',
        label: t('Messenger'),
        icon: isWhatsappProvider(account.provider_kind) ? 'tabler:brand-whatsapp' : 'tabler:brand-telegram',
        enabled: providerAccountEnabled(account),
        statusText: statusText(account),
        detail: t('Runtime-owned messaging service; setup continues through the managed route.'),
        canToggle: false,
        disabledReason: t('Messenger runtime toggles are not exposed through Settings yet.'),
        isBusy: false,
      })
    }

    if (isZoomProvider(account.provider_kind) || isYandexTelemostProvider(account.provider_kind)) {
      rows.push({
        id: 'meetings',
        label: t('Meetings'),
        icon: 'tabler:video',
        enabled: providerAccountEnabled(account),
        statusText: statusText(account),
        detail: t('Meeting integration is runtime-owned and managed through its provider flow.'),
        canToggle: false,
        disabledReason: t('Meeting runtime toggle is not exposed through Settings yet.'),
        isBusy: false,
      })
    }

    return rows
  }

  return {
    actionMessage: computed(() => store.actionMessage),
    activeMailAction,
    accounts,
    closeConnectWizard,
    connectWizardProviderId,
    connectWizardSelectedAccount,
    groups,
    isConnectWizardOpen,
    openConnectWizard,
    selectIntegration,
    selectedAccount,
    selectedAccountSummary,
    errorMessage: computed(() => store.errorMessage),
    hasAccounts,
    handleSaveSelectedAccountLabel,
    handleSelectedAccountLabelInput,
    handleDelete,
    handleExport,
    handleLogout,
    handleToggleSelectedAccount,
    handleToggleSelectedService,
    selectedMailSyncSettings,
    selectedMailSyncSettingsQuery,
  }
}

function matchesCalendarAccount(account: ProviderAccount, calendarAccount: CalendarAccount): boolean {
  const accountEmail = selectedAccountEmail(account).toLowerCase()
  const calendarEmail = calendarAccount.email?.toLowerCase() ?? ''
  return (
    calendarAccount.account_id === account.account_id ||
    calendarAccount.account_id === account.external_account_id ||
    calendarAccount.account_name === account.display_name ||
    (accountEmail.length > 0 && calendarEmail === accountEmail)
  )
}
