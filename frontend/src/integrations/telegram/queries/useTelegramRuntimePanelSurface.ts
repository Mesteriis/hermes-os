import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useRealtimeStatusStore } from '../../../shared/stores/realtimeStatus'
import {
  useTelegramAccountsQuery,
  useTelegramCapabilitiesQuery,
} from './useTelegramQuery'
import {
  useRestartTelegramRuntimeMutation,
  useStartTelegramRuntimeMutation,
  useStopTelegramRuntimeMutation,
  useTelegramRuntimeStatusQuery,
} from './useTelegramRuntimeQuery'

export function useTelegramRuntimePanelSurface() {
  const { t } = useI18n()
  const realtimeStatus = useRealtimeStatusStore()

  const selectedAccountIdState = ref<string | null>(null)
  const actionMessage = ref('')
  const actionError = ref('')

  const accountsQuery = useTelegramAccountsQuery()
  const capabilitiesQuery = useTelegramCapabilitiesQuery()
  const accounts = computed(() => accountsQuery.data.value ?? [])
  const selectedAccount = computed(() =>
    accounts.value.find((account) => account.account_id === selectedAccountIdState.value)
    ?? accounts.value[0]
    ?? null
  )
  const selectedAccountId = computed({
    get: () => selectedAccountIdState.value,
    set: (value: string | null) => {
      selectedAccountIdState.value = value
    },
  })
  const runtimeStatusQuery = useTelegramRuntimeStatusQuery(
    computed(() => selectedAccount.value?.account_id ?? null)
  )
  const startRuntimeMutation = useStartTelegramRuntimeMutation()
  const stopRuntimeMutation = useStopTelegramRuntimeMutation()
  const restartRuntimeMutation = useRestartTelegramRuntimeMutation()
  const isRuntimeBusy = computed(() =>
    startRuntimeMutation.isPending.value
    || stopRuntimeMutation.isPending.value
    || restartRuntimeMutation.isPending.value
  )

  watch(
    accounts,
    (items) => {
      if (!items.length) {
        selectedAccountIdState.value = null
        return
      }
      if (
        selectedAccountIdState.value
        && items.some((account) => account.account_id === selectedAccountIdState.value)
      ) {
        return
      }
      selectedAccountIdState.value = items[0]?.account_id ?? null
    },
    { immediate: true }
  )

  async function refreshRuntime() {
    await Promise.all([
      accountsQuery.refetch(),
      capabilitiesQuery.refetch(),
      runtimeStatusQuery.refetch(),
    ])
  }

  async function setTelegramRuntime(action: 'start' | 'stop' | 'restart') {
    const accountId = selectedAccount.value?.account_id
    if (!accountId || isRuntimeBusy.value) return
    actionMessage.value = ''
    actionError.value = ''
    try {
      const mutation = action === 'start'
        ? startRuntimeMutation
        : action === 'stop'
          ? stopRuntimeMutation
          : restartRuntimeMutation
      const status = await mutation.mutateAsync({ account_id: accountId })
      actionMessage.value = `Telegram runtime ${status.status}`
      await refreshRuntime()
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : t('Telegram runtime update failed')
    }
  }

  return {
    actionError,
    actionMessage,
    accounts,
    accountsQuery,
    capabilitiesQuery,
    isRuntimeBusy,
    realtimeStatus,
    refreshRuntime,
    runtimeStatusQuery,
    selectedAccount,
    selectedAccountId,
    selectedAccountIdState,
    setTelegramRuntime,
  }
}

export type TelegramRuntimePanelSurface = ReturnType<typeof useTelegramRuntimePanelSurface>
