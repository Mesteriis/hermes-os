import {
  useMailSyncSettingsQuery,
  useRunMailSyncNowMutation,
  useUpdateMailSyncSettingsMutation
} from '../../../integrations/mail/queries/runtimeQueries'
import type { useCommunicationsStore } from '../stores/communications'
import type { MailSyncSettingsUpdate } from '../types/communications'

type CommunicationsStore = ReturnType<typeof useCommunicationsStore>
type RefetchHandler = () => Promise<unknown>

type MailSyncRefetches = {
  refetchMailList: RefetchHandler
  refetchMailboxHealth: RefetchHandler
  refetchStateCounts: RefetchHandler
  refetchSyncStatuses: RefetchHandler
}

export function useMailSyncActions(store: CommunicationsStore, refetches: MailSyncRefetches) {
  const runMailSyncNowMutation = useRunMailSyncNowMutation()
  const updateMailSyncSettingsMutation = useUpdateMailSyncSettingsMutation()
  const {
    data: selectedMailSyncSettings,
    isLoading: isSyncSettingsLoading
  } = useMailSyncSettingsQuery(() => store.selectedMailAccountId || null)

  async function handleSyncNow() {
    const accountId = store.selectedMailAccountId
    if (!accountId) return
    store.setIsMailSyncBusy(true)
    store.setMailSyncStatusMessage('Syncing...')
    try {
      await runMailSyncNowMutation.mutateAsync(accountId)
      store.setMailSyncStatusMessage('Sync completed')
      await Promise.all([
        refetches.refetchMailList(),
        refetches.refetchStateCounts(),
        refetches.refetchSyncStatuses(),
        refetches.refetchMailboxHealth()
      ])
    } catch (e) {
      store.setMailSyncError(e instanceof Error ? e.message : 'Sync failed')
    } finally {
      store.setIsMailSyncBusy(false)
    }
  }

  function clearSyncStatus() {
    store.setMailSyncStatusMessage('')
    store.setMailSyncError('')
  }

  async function handleUpdateSyncSettings(settings: MailSyncSettingsUpdate) {
    const accountId = store.selectedMailAccountId
    if (!accountId) return
    store.setMailSyncStatusMessage('Saving sync settings...')
    store.setMailSyncError('')
    try {
      await updateMailSyncSettingsMutation.mutateAsync({ accountId, settings })
      store.setMailSyncStatusMessage('Sync settings saved')
      await refetches.refetchSyncStatuses()
    } catch (e) {
      store.setMailSyncError(e instanceof Error ? e.message : 'Sync settings update failed')
    }
  }

  async function loadInitialData() {
    await Promise.all([
      refetches.refetchSyncStatuses(),
      refetches.refetchMailboxHealth(),
      refetches.refetchStateCounts()
    ])
  }

  return {
    clearSyncStatus,
    handleUpdateSyncSettings,
    handleSyncNow,
    isSyncSettingsLoading,
    isSyncSettingsSaving: updateMailSyncSettingsMutation.isPending,
    selectedMailSyncSettings,
    loadInitialData
  }
}
