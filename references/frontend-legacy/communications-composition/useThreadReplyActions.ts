// Historical pre-clean-room thread reply orchestration. It is not part of the active client graph.
import { computed } from 'vue'
import {
  useSaveDraftMutation,
  useSendMailMutation
} from '../queries/useCommunicationsQuery'
import { useCommunicationActionNotifications } from '../queries/communicationActionNotifications'
import { buildComposeDraftPayload } from '../forms/composeDraftAutosave'
import {
  composeFormToSendRequest,
  threadReplyComposeForm
} from '../helpers/communicationPageModels'
import type { ThreadMessage } from '../types/communications'
import type { useCommunicationsStore } from '../stores/communications'

type CommunicationsStore = ReturnType<typeof useCommunicationsStore>

export function useThreadReplyActions(store: CommunicationsStore) {
  const saveDraftMutation = useSaveDraftMutation()
  const sendMailMutation = useSendMailMutation()
  const notifications = useCommunicationActionNotifications()
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
      notifications.success('Draft saved')
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : 'Save draft failed'
      store.setMailActionError(errorMessage)
      notifications.error('Draft save failed', errorMessage)
    }
  }

  async function handleSendThreadReply(message: ThreadMessage, bodyHtml: string, draftId: string) {
    if (!bodyHtml.trim()) return
    try {
      const form = threadReplyComposeForm(message, store.selectedMailAccountId, draftId || `draft-${Date.now()}`, bodyHtml)
      const result = await sendMailMutation.mutateAsync(composeFormToSendRequest(form))
      const status = `Sent via ${result.transport ?? 'provider'}`
      store.setMailActionStatus(status)
      notifications.success('Message queued', status)
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : 'Send failed'
      store.setMailActionError(errorMessage)
      notifications.error('Send failed', errorMessage)
    }
  }

  return {
    handleReplyToThreadMessage,
    handleSaveThreadReplyDraft,
    handleSendThreadReply,
    isThreadReplySending
  }
}
