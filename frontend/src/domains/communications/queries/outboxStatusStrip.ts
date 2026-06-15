import { computed, type MaybeRefOrGetter } from 'vue'
import {
  useOutboxQuery,
  useUndoOutboxMutation
} from './useCommunicationsQuery'

type OutboxStatusStripOptions = {
  onStatus?: (message: string) => void
  onError?: (message: string) => void
}

export function useOutboxStatusStrip(
  accountId: MaybeRefOrGetter<string | undefined>,
  options: OutboxStatusStripOptions = {}
) {
  const outboxQuery = useOutboxQuery(accountId)
  const undoMutation = useUndoOutboxMutation()
  const outboxItems = computed(() => outboxQuery.data.value ?? [])
  const outboxErrorMessage = computed(() => {
    if (!outboxQuery.error.value) return ''
    return outboxQuery.error.value instanceof Error
      ? outboxQuery.error.value.message
      : 'Failed to load outbox'
  })
  const isUndoingOutbox = computed(() => undoMutation.isPending.value)
  const hasMoreOutboxItems = computed(() => Boolean(outboxQuery.hasNextPage.value))
  const isLoadingMoreOutbox = computed(() => outboxQuery.isFetchingNextPage.value)

  async function undoOutbox(outboxId: string): Promise<void> {
    try {
      await undoMutation.mutateAsync(outboxId)
      options.onStatus?.('Send canceled')
      await outboxQuery.refetch()
    } catch (error) {
      options.onError?.(error instanceof Error ? error.message : 'Undo send failed')
    }
  }

  async function loadMoreOutboxItems(): Promise<void> {
    if (!outboxQuery.hasNextPage.value || outboxQuery.isFetchingNextPage.value) return
    try {
      await outboxQuery.fetchNextPage()
    } catch (error) {
      options.onError?.(error instanceof Error ? error.message : 'Failed to load outbox')
    }
  }

  async function prefetchMoreOutboxItems(): Promise<void> {
    if (!outboxQuery.hasNextPage.value || outboxQuery.isFetchingNextPage.value) return
    try {
      await outboxQuery.fetchNextPage()
    } catch {
      // Prefetch is opportunistic; explicit load-more reports user-facing errors.
    }
  }

  return {
    outboxItems,
    outboxErrorMessage,
    isOutboxLoading: outboxQuery.isLoading,
    isLoadingMoreOutbox,
    hasMoreOutboxItems,
    isUndoingOutbox,
    undoOutbox,
    loadMoreOutboxItems,
    prefetchMoreOutboxItems
  }
}
