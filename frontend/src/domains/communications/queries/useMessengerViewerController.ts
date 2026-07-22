import { computed, nextTick, ref, watch } from 'vue'
import type { MessengerAttachmentModel, MessengerConversationModel } from '../components/messengers/messengerElements'
import {
  messengerComposerCapabilityCanOpenFile,
  messengerComposerPlainText,
  type MessengerComposerCapability,
} from '../components/messengers/messengerComposer'
import { messengerConversationIsEmpty } from '../components/messengers/messengerElements'

export interface MessengerViewerControllerProps {
  conversation: MessengerConversationModel
  isActionRunning?: boolean
  isLoadingOlder?: boolean
}

export interface MessengerViewerControllerActions {
  openFilePicker: () => void
  submitMessage: (value: string) => void
  uploadFile: (file: File, caption: string) => void
  selectMessage: (messageId: string) => void
  downloadAttachment: (attachment: MessengerAttachmentModel) => void
  loadOlder: () => void
  messagesVisible: () => void
}

export function useMessengerViewerController(
  props: Readonly<MessengerViewerControllerProps>,
  actions: MessengerViewerControllerActions,
) {
  const pendingAttachment = ref<File | null>(null)
  const messagesContainer = ref<HTMLElement | null>(null)
  const historyScrollHeight = ref<number | null>(null)
  const isConversationEmpty = computed(() => messengerConversationIsEmpty(props.conversation))

  watch(
    () => props.conversation.id,
    () => {
      pendingAttachment.value = null
      historyScrollHeight.value = null
      void nextTick(() => {
        const container = messagesContainer.value
        if (container) container.scrollTop = container.scrollHeight
      })
    },
  )

  watch(
    [() => props.conversation.id, () => props.conversation.messages.length],
    () => {
      void nextTick(() => {
        const container = messagesContainer.value
        if (container && props.conversation.messages.length > 0 && container.clientHeight > 0) {
          actions.messagesVisible()
        }
      })
    },
    { immediate: true, flush: 'post' },
  )

  watch(
    () => props.isLoadingOlder,
    (isLoading, wasLoading) => {
      if (!wasLoading || isLoading || historyScrollHeight.value == null) return
      void nextTick(() => {
        const container = messagesContainer.value
        const previousHeight = historyScrollHeight.value
        historyScrollHeight.value = null
        if (container && previousHeight != null) {
          container.scrollTop += container.scrollHeight - previousHeight
        }
      })
    },
  )

  function handleComposerCapability(capability: MessengerComposerCapability): void {
    if (!messengerComposerCapabilityCanOpenFile(capability, Boolean(props.isActionRunning))) {
      return
    }
    actions.openFilePicker()
  }

  function handleFileChange(event: Event): void {
    if (!(event.target instanceof HTMLInputElement)) return
    const input = event.target
    const file = input.files?.[0]
    input.value = ''
    if (file) {
      pendingAttachment.value = file
    }
  }

  function handleComposerSubmit(value: string): void {
    if (props.isActionRunning) return

    const file = pendingAttachment.value
    if (!file) {
      actions.submitMessage(value)
      return
    }

    pendingAttachment.value = null
    actions.uploadFile(file, messengerComposerPlainText(value))
  }

  function handleMessageScroll(event: Event): void {
    if (!(event.currentTarget instanceof HTMLElement)) return
    const target = event.currentTarget
    if (target.scrollTop > 80 || props.isLoadingOlder || historyScrollHeight.value != null) return
    historyScrollHeight.value = target.scrollHeight
    actions.loadOlder()
  }

  function handleSelectMessage(messageId: string): void {
    actions.selectMessage(messageId)
  }

  function handleDownloadAttachment(attachment: MessengerAttachmentModel): void {
    actions.downloadAttachment(attachment)
  }

  return {
    pendingAttachment,
    messagesContainer,
    isConversationEmpty,
    handleComposerCapability,
    handleFileChange,
    handleComposerSubmit,
    handleMessageScroll,
    handleSelectMessage,
    handleDownloadAttachment,
  }
}
