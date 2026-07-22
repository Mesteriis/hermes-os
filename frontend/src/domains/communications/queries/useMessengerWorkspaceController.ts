import { ref } from 'vue'
import type { MessengerAttachmentModel, MessengerConversationModel, MessengerListItemModel } from '../components/messengers/messengerElements'
import type { MessengerConversationRuntimeAction } from '@/shared/communications/types/messengerRuntimeActions'

type MessengerWorkspaceControllerProps = {
  isListLoading?: boolean
  isListRefreshing?: boolean
  listError?: string
  items: readonly MessengerListItemModel[]
  conversation: MessengerConversationModel
}

export interface MessengerWorkspaceControllerActions {
  refresh: () => void
  selectConversation: (item: MessengerListItemModel) => void
  conversationAction: (action: MessengerConversationRuntimeAction) => void
  selectMessage: (messageId: string) => void
  submit: (value: string) => void
  uploadFile: (file: File, caption: string) => void
  downloadAttachment: (attachment: MessengerAttachmentModel) => void
  loadOlder: () => void
  messagesVisible: () => void
}

export function useMessengerWorkspaceController(
  _props: Readonly<MessengerWorkspaceControllerProps>,
  actions: MessengerWorkspaceControllerActions,
) {
  void _props
  const isInspectorVisible = ref(true)

  function handleToggleInspector(): void {
    isInspectorVisible.value = !isInspectorVisible.value
  }

  function handleRefresh(): void {
    actions.refresh()
  }

  function handleSelectConversation(item: MessengerListItemModel): void {
    actions.selectConversation(item)
  }

  function handleConversationAction(action: MessengerConversationRuntimeAction): void {
    actions.conversationAction(action)
  }

  function handleSelectMessage(messageId: string): void {
    actions.selectMessage(messageId)
  }

  function handleSubmit(value: string): void {
    actions.submit(value)
  }

  function handleUploadFile(file: File, caption: string): void {
    actions.uploadFile(file, caption)
  }

  function handleDownloadAttachment(attachment: MessengerAttachmentModel): void {
    actions.downloadAttachment(attachment)
  }

  function handleLoadOlder(): void {
    actions.loadOlder()
  }

  function handleMessagesVisible(): void {
    actions.messagesVisible()
  }

  return {
    isInspectorVisible,
    handleToggleInspector,
    handleRefresh,
    handleSelectConversation,
    handleConversationAction,
    handleSelectMessage,
    handleSubmit,
    handleUploadFile,
    handleDownloadAttachment,
    handleLoadOlder,
    handleMessagesVisible,
  }
}
