import { computed, ref } from 'vue'
import { useSettingsStore } from '../stores/settings'
import {
  buildMaintenanceActionRows,
  buildMaintenanceBackupRows,
  buildMaintenanceInventoryRows,
  buildMaintenanceSummaryTiles,
  formatBytes,
  totalInventoryBytes
} from '../components/maintenanceSettingsPresentation'
import {
  useMaintenanceOverviewQuery,
  useRunMaintenanceActionMutation
} from './useMaintenanceQuery'

export function useMaintenanceSettingsSurface() {
  const store = useSettingsStore()
  const overviewQuery = useMaintenanceOverviewQuery()
  const runActionMutation = useRunMaintenanceActionMutation()
  const selectedActionId = ref<string | null>(null)
  const confirmationDraft = ref('')

  const overview = computed(() => overviewQuery.data.value ?? null)
  const inventoryItems = computed(() => overview.value?.inventory ?? [])
  const backupItems = computed(() => overview.value?.backups ?? [])
  const actionItems = computed(() => overview.value?.actions ?? [])
  const inventoryRows = computed(() => buildMaintenanceInventoryRows(inventoryItems.value))
  const backupRows = computed(() => buildMaintenanceBackupRows(backupItems.value))
  const actionRows = computed(() => buildMaintenanceActionRows(actionItems.value))
  const summaryTiles = computed(() =>
    buildMaintenanceSummaryTiles(inventoryItems.value, backupItems.value)
  )
  const totalSizeLabel = computed(() => formatBytes(totalInventoryBytes(inventoryItems.value)))
  const selectedAction = computed(() =>
    actionRows.value.find((action) => action.id === selectedActionId.value) ?? null
  )
  const canRunSelectedAction = computed(() => {
    const action = selectedAction.value
    if (!action?.enabled || runActionMutation.isPending.value) return false
    if (!action.requires_confirmation) return true
    return confirmationDraft.value === action.confirmation_phrase
  })
  const isLoading = computed(() => overviewQuery.isLoading.value)
  const isBusy = computed(() => runActionMutation.isPending.value)
  const errorMessage = computed(() => {
    if (!overviewQuery.isError.value) return ''
    return overviewQuery.error.value instanceof Error
      ? overviewQuery.error.value.message
      : 'Maintenance overview request failed'
  })

  function handleRefresh() {
    void overviewQuery.refetch()
  }

  function handleSelectAction(actionId: string) {
    selectedActionId.value = actionId
    confirmationDraft.value = ''
  }

  function handleConfirmationInput(value: string) {
    confirmationDraft.value = value
  }

  async function handleRunSelectedAction() {
    const action = selectedAction.value
    if (!action || !canRunSelectedAction.value) return
    try {
      const result = await runActionMutation.mutateAsync({
        actionId: action.id,
        request: {
          confirmation: action.requires_confirmation ? confirmationDraft.value : undefined
        }
      })
      store.setActionMessage(result.message)
      confirmationDraft.value = ''
    } catch (error) {
      store.setError(error instanceof Error ? error.message : 'Maintenance action failed')
    }
  }

  return {
    actionRows,
    backupRows,
    canRunSelectedAction,
    confirmationDraft,
    errorMessage,
    handleConfirmationInput,
    handleRefresh,
    handleRunSelectedAction,
    handleSelectAction,
    inventoryRows,
    isBusy,
    isLoading,
    selectedAction,
    selectedActionId,
    summaryTiles,
    totalSizeLabel
  }
}

export type MaintenanceSettingsSurface = ReturnType<typeof useMaintenanceSettingsSurface>
