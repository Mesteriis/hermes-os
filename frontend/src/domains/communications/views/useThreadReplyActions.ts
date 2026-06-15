import { computed } from 'vue'
import {
  useSaveDraftMutation,
  useSendMailMutation
} from '../queries/useCommunicationsQuery'
import { buildComposeDraftPayload } from '../forms/composeDraftAutosave'
import {
  composeFormToSendRequest,
  threadReplyComposeForm
} from '../helpers/mailPageModels'
import type { ThreadMessage } from '../types/communications'
import type { useCommunicationsStore } from '../stores/communications'

type CommunicationsStore = ReturnType<typeof useCommunicationsStore>

export function useThreadReplyActions(store: CommunicationsStore) {
  const saveDraftMutation = useSaveDraftMutation()
  const sendMailMutation = useSendMailMutation()
  const isThreadReplySending = computed(() => sendMailMutation.isPending.value)

  function handleReplyToThreadMessage(message: ThreadMessage, bodyHtml: string, draftId: string) {
    store.openCompose(threadReplyComposeForm(message, store.selectedMailAccountId, draftId || `draft-${Date.now()}`, bodyHtml))
  }

  async function handleSaveThreadReplyDraft(message: ThreadMessage, bodyHtml: string, draftId: string) {
    if (!bodyHtml.trim()) return
    try {
      await saveDraftMutation.mutateAsync(buildComposeDraftPayload(
        threadReplyComposeForm(message, store.selectedMailAccountId, draftId, bodyHtml)
      ))
      store.setMailActionStatus('Draft saved')
    } catch (e) {
      store.setMailActionError(e instanceof Error ? e.message : 'Save draft failed')
    }
  }

  async function handleSendThreadReply(message: ThreadMessage, bodyHtml: string, draftId: string) {
    if (!bodyHtml.trim()) return
    try {
      const form = threadReplyComposeForm(message, store.selectedMailAccountId, draftId || `draft-${Date.now()}`, bodyHtml)
      const result = await sendMailMutation.mutateAsync(composeFormToSendRequest(form))
      store.setMailActionStatus(`Sent via ${result.transport ?? 'provider'}`)
    } catch (e) {
      store.setMailActionError(e instanceof Error ? e.message : 'Send failed')
    }
  }

  return {
    handleReplyToThreadMessage,
    handleSaveThreadReplyDraft,
    handleSendThreadReply,
    isThreadReplySending
  }
}
