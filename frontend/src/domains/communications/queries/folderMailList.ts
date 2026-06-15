import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import { useFolderMessagesQuery } from './useCommunicationsQuery'
import type { CommunicationMessageSummary } from '../types/communications'
import type { FolderMessage } from '../types/folders'

export function folderMessagesToMailSummaries(
  folderMessages: FolderMessage[]
): CommunicationMessageSummary[] {
  return folderMessages.map((message) => ({
    message_id: message.message_id,
    raw_record_id: '',
    account_id: message.account_id,
    provider_record_id: '',
    subject: message.subject,
    sender: message.sender,
    recipients: [],
    body_text_preview: '',
    occurred_at: message.occurred_at,
    projected_at: message.projected_at,
    channel_kind: 'email',
    conversation_id: null,
    sender_display_name: null,
    delivery_state: 'folder',
    workflow_state: message.workflow_state,
    importance_score: null,
    ai_category: null,
    ai_summary: null,
    ai_summary_generated_at: null,
    message_metadata: {
      folder_id: message.folder_id,
      folder_added_at: message.added_at
    },
    attachment_count: message.attachment_count,
    local_state: message.local_state,
    local_state_changed_at: null
  }))
}

export function useFolderMailList(folderId: MaybeRefOrGetter<string | null | undefined>) {
  const activeFolderId = computed(() => toValue(folderId)?.trim() || null)
  const query = useFolderMessagesQuery(
    () => activeFolderId.value,
    () => Boolean(activeFolderId.value)
  )
  const messages = computed(() => folderMessagesToMailSummaries(query.data.value ?? []))
  const errorMessage = computed(() => {
    if (!query.error.value) return ''
    return query.error.value instanceof Error
      ? query.error.value.message
      : 'Folder messages request failed'
  })

  return {
    ...query,
    messages,
    errorMessage
  }
}
