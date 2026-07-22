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
import {
  accountCredentialRequiresReauthorization,
  defaultProviderIdFromAccount,
  isMailProvider,
  matchesCalendarAccount,
  providerAccountEnabled,
  selectedAccountEmail,
  toIntegrationServiceRows,
  canOpenSetupAction,
  selectedInspectorActionLabel,
  nextStepLabel,
  toIntegrationAccountRow,
  toIntegrationGroups,
  type AccountServiceRow,
  type IntegrationGroup,
  type SelectedIntegrationSummary,
} from './integrationAccountPresentation'
import {
  enableContactsBidirectional,
  runContactsSyncNow,
  toggleContactsService,
} from './integrationContactsActions'
import {
  deleteMailAccount,
  exportMailAccount,
  logoutMailAccount,
  saveAccountLabel,
} from './integrationAccountActions'
import { toggleCalendarService } from './integrationCalendarActions'
import { toggleMailService } from './integrationMailActions'
import { toggleSelectedAccount } from './integrationAccountToggleActions'
import {
  runSelectedIntegrationService,
  toggleSelectedIntegrationService
} from './integrationServiceActions'
import { downloadJsonFile } from '../../../shared/file/downloadJson'

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

  const contactsActionDependencies = {
    t,
    setActiveAccount: (accountId: string | null) => {
      activeMailAction.value = accountId
    },
    clearMessages: () => store.clearMessages(),
    setActionMessage: (message: string) => store.setActionMessage(message),
    setError: (message: string) => store.setError(message),
    updateProviderAccount: updateProviderAccountMutation.mutateAsync,
    runAddressBookSyncNow: runAddressBookSyncNowMutation.mutateAsync,
  }
  const accountActionDependencies = {
    t,
    setActiveAccount: (accountId: string | null) => {
      activeMailAction.value = accountId
    },
    clearMessages: () => store.clearMessages(),
    setActionMessage: (message: string) => store.setActionMessage(message),
    setError: (message: string) => store.setError(message),
    clearSelectedAccount: (accountId: string) => {
      if (store.selectedIntegrationId === accountId) {
        store.selectIntegration(null)
      }
    },
    updateProviderAccount: updateProviderAccountMutation.mutateAsync,
    exportMailAccount: exportMailMutation.mutateAsync,
    logoutMailAccount: logoutMailMutation.mutateAsync,
    deleteMailAccount: deleteMailMutation.mutateAsync,
    downloadJsonFile,
  }
  const mailActionDependencies = {
    t,
    setActiveAccount: (accountId: string | null) => {
      activeMailAction.value = accountId
    },
    clearMessages: () => store.clearMessages(),
    setActionMessage: (message: string) => store.setActionMessage(message),
    setError: (message: string) => store.setError(message),
    updateMailSyncSettings: updateMailSyncSettingsMutation.mutateAsync,
  }
  const calendarActionDependencies = {
    t,
    setActiveAccount: (accountId: string | null) => {
      activeMailAction.value = accountId
    },
    clearMessages: () => store.clearMessages(),
    setActionMessage: (message: string) => store.setActionMessage(message),
    setError: (message: string) => store.setError(message),
    updateCalendarAccount: updateCalendarAccountMutation.mutateAsync,
  }
  const accountToggleDependencies = {
    t,
    openConnectWizard,
    logoutMailAccount: (accountId: string) => logoutMailAccount(accountId, accountActionDependencies),
    setError: (message: string) => store.setError(message),
  }

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

  const groups = computed<IntegrationGroup[]>(() =>
    toIntegrationGroups(accounts.value, store.selectedIntegrationId, t)
  )
  const hasAccounts = computed(() => accounts.value.length > 0)

  const selectedAccountSummary = computed<SelectedIntegrationSummary | null>(() => {
    if (!selectedAccount.value) return null
    const account = selectedAccount.value
    const accountEnabled = providerAccountEnabled(account)
    const accountRow = toIntegrationAccountRow(account, store.selectedIntegrationId, t)
    return {
      ...accountRow,
      email: selectedAccountEmail(account),
      selectedInspectorActionLabel: selectedInspectorActionLabel(account, t),
      nextStepLabel: nextStepLabel(account.provider_kind, t),
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
      services: toIntegrationServiceRows({
        account,
        selectedMailSyncSettings: selectedMailSyncSettings.value,
        calendarAccount: selectedCalendarAccount.value,
        activeMailActionAccountId: activeMailAction.value,
        isMailSyncUpdatePending: updateMailSyncSettingsMutation.isPending.value,
        isCalendarUpdatePending: updateCalendarAccountMutation.isPending.value,
        isProviderUpdatePending: updateProviderAccountMutation.isPending.value,
        isRunAddressBookSyncNowPending: runAddressBookSyncNowMutation.isPending.value,
        t,
      }),
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
    await saveAccountLabel(selectedAccount.value, selectedAccountLabelDraft.value, accountActionDependencies)
  }

  async function handleExport(accountId: string) {
    await exportMailAccount(accountId, accountActionDependencies)
  }

  async function handleLogout(accountId: string) {
    await logoutMailAccount(accountId, accountActionDependencies)
  }

  async function handleDelete(accountId: string) {
    await deleteMailAccount(accountId, accountActionDependencies)
  }

  async function handleToggleSelectedAccount(enabled: boolean) {
    await toggleSelectedAccount(selectedAccount.value, enabled, accountToggleDependencies)
  }

  async function handleToggleSelectedService(serviceId: AccountServiceRow['id'], enabled: boolean) {
    await toggleSelectedIntegrationService(selectedAccount.value, serviceId, enabled, {
      selectedCalendarAccount: selectedCalendarAccount.value,
      toggleMail: handleToggleMailService,
      toggleCalendar: (account, value) => toggleCalendarService(account, value, calendarActionDependencies),
      toggleContacts: handleToggleContactsService,
      runContactsSync: (account) => runContactsSyncNow(account, contactsActionDependencies),
      setError: (message) => store.setError(message),
      unsupportedMessage: t('This service does not expose a settings toggle contract yet.')
    })
  }

  async function handleToggleMailService(account: ProviderAccount, enabled: boolean) {
    await toggleMailService(account, enabled, selectedMailSyncSettings.value, mailActionDependencies)
  }

  async function handleToggleCalendarService(enabled: boolean) {
    await toggleCalendarService(selectedCalendarAccount.value, enabled, calendarActionDependencies)
  }

  async function handleToggleContactsService(account: ProviderAccount, enabled: boolean) {
    await toggleContactsService(account, enabled, contactsActionDependencies)
  }

  async function handleEnableSelectedContactsBidirectional() {
    const account = selectedAccount.value
    if (!account) return
    await enableContactsBidirectional(account, contactsActionDependencies)
  }

  async function handleRunSelectedServiceNow(serviceId: AccountServiceRow['id']) {
    await runSelectedIntegrationService(selectedAccount.value, serviceId, {
      selectedCalendarAccount: selectedCalendarAccount.value,
      toggleMail: handleToggleMailService,
      toggleCalendar: (account, value) => toggleCalendarService(account, value, calendarActionDependencies),
      toggleContacts: handleToggleContactsService,
      runContactsSync: (account) => runContactsSyncNow(account, contactsActionDependencies),
      setError: (message) => store.setError(message),
      unsupportedMessage: t('This service does not expose a manual sync action yet.')
    })
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
