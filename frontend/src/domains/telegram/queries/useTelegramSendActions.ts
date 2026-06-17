import { ref } from 'vue'
import { useReplyTelegramMessageMutation, useSendTelegramMessageMutation } from './useTelegramQuery'
import { useTelegramMediaUploadMutation } from './useTelegramMediaUploadQuery'
import type { TelegramChat, TelegramMessage } from '../types/telegram'

export function useTelegramSendActions(
  getSelectedChat: () => TelegramChat | null,
  getIsBusy: () => boolean,
  getManualSendText: () => string,
  callbacks: {
    setActionSubmitting: (v: boolean) => void
    setActionMessage: (v: string) => void
    setError: (v: string) => void
    resetSendForm: () => void
    setSelectedChatId: (id: string) => void
  }
) {
  const replyTo = ref<TelegramMessage | null>(null)
  const sendMessageMutation = useSendTelegramMessageMutation()
  const replyMessageMutation = useReplyTelegramMessageMutation()
  const mediaUploadMutation = useTelegramMediaUploadMutation()

  async function sendOrReply() {
    const chat = getSelectedChat()
    if (getIsBusy() || !chat) return
    callbacks.setActionSubmitting(true)
    callbacks.setActionMessage('')
    callbacks.setError('')
    try {
      const currentReplyTo = replyTo.value
      if (currentReplyTo?.provider_message_id) {
        const result = await replyMessageMutation.mutateAsync({
          message_id: currentReplyTo.message_id,
          account_id: chat.account_id,
          provider_chat_id: chat.provider_chat_id,
          reply_to_provider_message_id: currentReplyTo.provider_message_id,
          text: getManualSendText(),
        })
        replyTo.value = null
        callbacks.setSelectedChatId(result.provider_chat_id)
        callbacks.setActionMessage(`Reply ${result.status}`)
      } else {
        const result = await sendMessageMutation.mutateAsync({
          account_id: chat.account_id,
          provider_chat_id: chat.provider_chat_id,
          text: getManualSendText(),
        })
        callbacks.setSelectedChatId(result.provider_chat_id)
        callbacks.setActionMessage(`Telegram message ${result.status}`)
      }
      callbacks.resetSendForm()
    } catch (err) {
      callbacks.setError(err instanceof Error ? err.message : String(err))
    } finally {
      callbacks.setActionSubmitting(false)
    }
  }

  async function uploadMedia(file: File) {
    const chat = getSelectedChat()
    if (getIsBusy() || !chat) return
    callbacks.setActionSubmitting(true)
    callbacks.setActionMessage('')
    callbacks.setError('')
    try {
      const result = await mediaUploadMutation.mutateAsync({
        accountId: chat.account_id,
        providerChatId: chat.provider_chat_id,
        file,
        caption: getManualSendText().trim() || undefined
      })
      callbacks.setSelectedChatId(result.provider_chat_id)
      callbacks.setActionMessage(`Telegram media upload ${result.status}`)
      callbacks.resetSendForm()
    } catch (err) {
      callbacks.setError(err instanceof Error ? err.message : String(err))
    } finally {
      callbacks.setActionSubmitting(false)
    }
  }

  return { replyTo, sendOrReply, uploadMedia }
}
