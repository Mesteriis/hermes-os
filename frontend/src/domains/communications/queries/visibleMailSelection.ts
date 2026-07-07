import type { Ref } from 'vue'
import type { useCommunicationsStore } from '../stores/communications'
import type { CommunicationMessageSummary } from '../types/communications'

type CommunicationsStore = ReturnType<typeof useCommunicationsStore>

type VisibleMailSelectionSurface = {
  visibleMailList: Ref<CommunicationMessageSummary[]>
  store: CommunicationsStore
  handleSelectMessage: (index: number) => void
}

export function handleVisibleMailItemIdsChange(
  pageSurface: VisibleMailSelectionSurface,
  itemIds: string[]
): void {
  const selectedId = pageSurface.store.selectedCommunicationMessageId
  if (selectedId && itemIds.includes(selectedId)) return

  if (itemIds.length === 0) {
    pageSurface.store.clearSelectedMessageContext()
    return
  }

  const messageIndex = pageSurface.visibleMailList.value.findIndex(
    (message) => message.message_id === itemIds[0]
  )
  if (messageIndex < 0) return
  pageSurface.handleSelectMessage(messageIndex)
}
