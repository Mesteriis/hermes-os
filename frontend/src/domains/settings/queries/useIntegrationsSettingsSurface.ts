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
  useRunAddressBookSyncNowMutation,
  useUpdateCalendarAccountMutation,
  useUpdateProviderAccountMutation,
} from './useSettingsQuery'
import { useSettingsStore } from '../stores/settings'
import type { ProviderAccount } from '../types/settings'
import { communicationProviderBrand, providerBrandClass } from '../components/providerBranding'
import {
  accountConfigString,
  accountContactsRemoteWriteEnabled,
  accountContactsSyncEnabled,
  accountContactsSyncDirection,
  accountCredentialRequiresReauthorization,
  accountHasGoogleContactsWriteScope,
  accountSupportsContacts,
  defaultProviderIdFromAccount,
  downloadJson,
  isExceptionOnlyProvider,
  isExceptionRouteProvider,
  isMailProvider,
  isManagedRuntimeProvider,
  isTelegramProvider,
  isWhatsappProvider,
  isYandexTelemostProvider,
  isZoomProvider,
  isZulipProvider,
  matchesCalendarAccount,
  providerAccountEnabled,
  providerDisplayName,
  providerLabel,
  selectedAccountEmail,
  supportsCalendar,
  type AccountServiceRow,
  type IntegrationAccountRow,
  type IntegrationGroup,
  type SelectedIntegrationSummary,
} from './integrationAccountPresentation'

export type { AccountServiceRow, IntegrationAccountRow, IntegrationGroup, SelectedIntegrationSummary } from './integrationAccountPresentation'

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
  const runAddressBookSyncNowMutation = useRunAddressBookSyncNowMutation()

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

  const toIntegrationAccountRow = (account: ProviderAccount): IntegrationAccountRow => {
    const brand = communicationProviderBrand(account.provider_kind)
    return {
      account,
      displayName: providerDisplayName(account),
      icon: brand.icon,
      iconTone: providerBrandClass(brand),
      providerLabel: providerLabel(account.provider_kind),
      flowLabel: providerFlowLabel(account.provider_kind),
      statusText: statusText(account),
      statusClass: statusClass(account),
      isSelected: store.selectedIntegrationId === account.account_id,
    }
  }

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
    const brand = communicationProviderBrand(account.provider_kind)
    return {
      account,
      displayName: providerDisplayName(account),
      icon: brand.icon,
      iconTone: providerBrandClass(brand),
      providerLabel: providerLabel(account.provider_kind),
      flowLabel: providerFlowLabel(account.provider_kind),
      email: selectedAccountEmail(account),
      statusText: statusText(account),
      nextStepLabel: nextStepLabel(account.provider_kind),
      selectedInspectorActionLabel: selectedInspectorActionLabel(account),
      canManageMail: isMailProvider(account.provider_kind),
      canOpenSetupAction: canOpenSetupAction(account),
      canRecoverExpiredCredential: accountCredentialRequiresReauthorization(account),
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

  function openCredentialRecovery() {
    openConnectWizard(null)
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
      const result = await deleteMailMutation.mutateAsync(accountId)
      if (store.selectedIntegrationId === accountId) {
        store.selectIntegration(null)
      }
      store.setActionMessage(
        result.vault_deleted_secret_refs.length > 0
          ? t('Mail account deleted from Hermes and vault')
          : t('Mail account deleted')
      )
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

    if (serviceId === 'contacts') {
      await handleToggleContactsService(account, enabled)
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
          failure_threshold: current.failure_threshold ?? 3,
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

  async function handleToggleContactsService(account: ProviderAccount, enabled: boolean) {
    if (!accountSupportsContacts(account)) {
      store.setError(t('Contacts are not provided by this integration.'))
      return
    }
    if (accountConfigString(account, 'address_book_sync_unsupported_reason')) {
      store.setError(t('Contacts sync is disabled for this account because the provider adapter is not available.'))
      return
    }

    activeMailAction.value = account.account_id
    store.clearMessages()
    try {
      await updateProviderAccountMutation.mutateAsync({
        accountId: account.account_id,
        update: {
          address_book_sync_enabled: enabled,
        },
      })
      store.setActionMessage(enabled ? t('Contacts sync enabled') : t('Contacts sync paused'))
    } catch (error) {
      store.setError(error instanceof Error ? error.message : t('Contacts sync update failed'))
    } finally {
      activeMailAction.value = null
    }
  }

  async function handleEnableSelectedContactsBidirectional() {
    const account = selectedAccount.value
    if (!account) return
    if (!accountSupportsContacts(account)) {
      store.setError(t('Contacts are not provided by this integration.'))
      return
    }
    if (accountConfigString(account, 'address_book_sync_unsupported_reason')) {
      store.setError(t('Contacts sync is disabled for this account because the provider adapter is not available.'))
      return
    }

    activeMailAction.value = account.account_id
    store.clearMessages()
    try {
      await updateProviderAccountMutation.mutateAsync({
        accountId: account.account_id,
        update: {
          address_book_sync_enabled: true,
          address_book_sync_direction: 'bidirectional',
          address_book_remote_write_enabled: accountHasGoogleContactsWriteScope(account),
        },
      })
      store.setActionMessage(
        accountHasGoogleContactsWriteScope(account)
          ? t('Contacts two-way sync enabled')
          : t('Contacts two-way sync is prepared; reconnect with Contacts write scope to push changes.')
      )
    } catch (error) {
      store.setError(error instanceof Error ? error.message : t('Contacts sync update failed'))
    } finally {
      activeMailAction.value = null
    }
  }

  async function handleRunSelectedServiceNow(serviceId: AccountServiceRow['id']) {
    const account = selectedAccount.value
    if (!account) return

    if (serviceId !== 'contacts') {
      store.setError(t('This service does not expose a manual sync action yet.'))
      return
    }
    if (!accountSupportsContacts(account)) {
      store.setError(t('Contacts are not provided by this integration.'))
      return
    }
    if (accountConfigString(account, 'address_book_sync_unsupported_reason')) {
      store.setError(t('Contacts sync is disabled for this account because the provider adapter is not available.'))
      return
    }

    activeMailAction.value = account.account_id
    store.clearMessages()
    try {
      const result = await runAddressBookSyncNowMutation.mutateAsync(account.account_id)
      store.setActionMessage(
        t('Address book sync finished: {provider} provider entries, {local} local entries.', {
          provider: String(result.provider_entries_upserted),
          local: String(result.local_entries_pushed),
        })
      )
    } catch (error) {
      store.setError(error instanceof Error ? error.message : t('Address book sync failed'))
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
      canRunNow: false,
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
      canRunNow: false,
      disabledReason: calendarAccount ? undefined : t('Calendar account endpoint has no linked account yet.'),
        isBusy: updateCalendarAccountMutation.isPending.value,
      })
    }

    const contactsSupported = accountSupportsContacts(account)
    const contactsEnabled = contactsSupported && accountContactsSyncEnabled(account)
    const contactsUnsupportedReason = accountConfigString(account, 'address_book_sync_unsupported_reason')
    const contactsDirection = accountContactsSyncDirection(account)
    const contactsRemoteWriteEnabled = accountContactsRemoteWriteEnabled(account)
    const contactsCanWrite = account.provider_kind === 'gmail' && accountHasGoogleContactsWriteScope(account)
    const contactsDetail = contactsSupported
      ? contactsUnsupportedReason
        ? t('Contacts sync is disabled for this account because the provider adapter is not available.')
        : contactsDirection === 'bidirectional'
          ? contactsRemoteWriteEnabled
            ? t('Two-way sync with provider contacts is enabled.')
            : contactsCanWrite
              ? t('Two-way sync is selected, but provider write is paused.')
              : t('Two-way sync is selected, but this account needs Contacts write permission before Hermes can push changes.')
          : t('Contacts sync reads provider contacts into Personas. Local changes are not pushed.')
      : t('Contacts are not provided by this integration.')
    rows.push({
      id: 'contacts',
      label: t('Contacts'),
      icon: 'tabler:address-book',
      enabled: contactsEnabled,
      statusText: contactsSupported
        ? contactsEnabled
          ? contactsDirection === 'bidirectional'
            ? t('Two-way sync')
            : t('Read-only sync')
          : contactsUnsupportedReason
            ? t('Not supported')
            : t('Sync paused')
        : t('Not provided'),
      detail: contactsDetail,
      canToggle: contactsSupported && !contactsUnsupportedReason,
      canRunNow: contactsSupported && !contactsUnsupportedReason,
      runNowLabel: t('Sync now'),
      modeActionLabel: contactsDirection === 'bidirectional' ? undefined : t('Enable two-way'),
      canRunModeAction: contactsSupported && !contactsUnsupportedReason && contactsDirection !== 'bidirectional',
      disabledReason: contactsSupported
        ? contactsUnsupportedReason
          ? t('Contacts sync is disabled for this account because the provider adapter is not available.')
          : undefined
        : t('Contacts are not provided by this integration.'),
      isBusy: (updateProviderAccountMutation.isPending.value || runAddressBookSyncNowMutation.isPending.value) && activeMailAction.value === account.account_id,
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
        canRunNow: false,
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
        canRunNow: false,
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
    openCredentialRecovery,
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
    handleRunSelectedServiceNow,
    handleEnableSelectedContactsBidirectional,
    selectedMailSyncSettings,
    selectedMailSyncSettingsQuery,
  }
}
