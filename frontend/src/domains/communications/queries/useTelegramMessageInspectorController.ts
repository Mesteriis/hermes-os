import { computed, ref, watch, type Ref } from 'vue'
import type { TelegramMessage } from '@/shared/communications/types/telegram'
import { telegramForwardTargets } from './telegramWorkspacePresentation'
import {
  buildTelegramDeleteRequest,
  buildTelegramEditRequest,
  buildTelegramMarkReadRequest,
  buildTelegramPinRequest,
  buildTelegramReactionRequest,
  buildTelegramReactionMutationRequest,
  buildTelegramForwardRequest,
  buildTelegramReplyRequest,
  buildTelegramRestoreRequest,
  telegramMessageCommandId,
} from '../components/messengers/telegramMessageInspectorActions'
import {
  useAddTelegramReactionMutation,
  useDeleteTelegramMessageMutation,
  useEditTelegramMessageMutation,
  useForwardTelegramMessageMutation,
  useMarkReadTelegramMessageMutation,
  usePinTelegramMessageMutation,
  useReplyTelegramMessageMutation,
  useRestoreTelegramMessageMutation,
  useRemoveTelegramReactionMutation,
  useTelegramChatsQuery,
  useTelegramForwardChainQuery,
  useTelegramMessageReactionsQuery,
  useTelegramMessageTombstonesQuery,
  useTelegramMessageVersionsQuery,
  useTelegramRawMessageEvidenceQuery,
  useTelegramReplyChainQuery,
} from './telegramBusinessQueries'

export function useTelegramMessageInspectorController(telegramMessage: Ref<TelegramMessage>) {
  const editText = ref(telegramMessage.value.text)
  const forwardTargetChatId = ref('')
  const replyText = ref('')
  const reactionEmoji = ref('👍')
  const status = ref('')
  const error = ref('')
  const messageId = computed(() => telegramMessage.value.message_id)
  const accountId = computed(() => telegramMessage.value.account_id)
  const forwardTargetsQuery = useTelegramChatsQuery(accountId, 200)
  const forwardTargets = computed(() =>
    telegramForwardTargets(forwardTargetsQuery.data.value ?? [], telegramMessage.value.provider_chat_id)
  )
  const reactionsQuery = useTelegramMessageReactionsQuery(messageId)
  const versionsQuery = useTelegramMessageVersionsQuery(messageId)
  const tombstonesQuery = useTelegramMessageTombstonesQuery(messageId)
  const replyChainQuery = useTelegramReplyChainQuery(messageId)
  const forwardChainQuery = useTelegramForwardChainQuery(messageId)
  const rawEvidenceQuery = useTelegramRawMessageEvidenceQuery(messageId)
  const deleteMutation = useDeleteTelegramMessageMutation()
  const editMutation = useEditTelegramMessageMutation()
  const forwardMutation = useForwardTelegramMessageMutation()
  const markReadMutation = useMarkReadTelegramMessageMutation()
  const pinMutation = usePinTelegramMessageMutation()
  const replyMutation = useReplyTelegramMessageMutation()
  const restoreMutation = useRestoreTelegramMessageMutation()
  const addReactionMutation = useAddTelegramReactionMutation()
  const removeReactionMutation = useRemoveTelegramReactionMutation()
  const isRunning = computed(() =>
    deleteMutation.isPending.value
    || editMutation.isPending.value
    || forwardMutation.isPending.value
    || markReadMutation.isPending.value
    || pinMutation.isPending.value
    || replyMutation.isPending.value
    || restoreMutation.isPending.value
    || addReactionMutation.isPending.value
    || removeReactionMutation.isPending.value
  )

  watch(messageId, () => {
    editText.value = telegramMessage.value.text
    forwardTargetChatId.value = ''
    replyText.value = ''
    status.value = ''
    error.value = ''
  })

  async function run(label: string, command: () => Promise<unknown>): Promise<void> {
    status.value = ''
    error.value = ''
    try {
      await command()
      status.value = label
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : 'Telegram message command failed.'
    }
  }

  function commandId(): string {
    return telegramMessageCommandId()
  }

  function editMessage(): Promise<void> {
    const newText = editText.value.trim()
    if (!newText || newText === telegramMessage.value.text.trim()) return Promise.resolve()
    return run('Telegram edit command queued.', () => editMutation.mutateAsync(
      buildTelegramEditRequest(telegramMessage.value, newText, commandId())
    ))
  }

  function replyToMessage(): Promise<void> {
    const text = replyText.value.trim()
    if (!text) return Promise.resolve()
    return run('Telegram reply command queued.', async () => {
      await replyMutation.mutateAsync(buildTelegramReplyRequest(telegramMessage.value, text))
      replyText.value = ''
    })
  }

  function forwardMessage(): Promise<void> {
    const providerChatId = forwardTargetChatId.value.trim()
    if (!providerChatId) return Promise.resolve()
    return run('Telegram forward command queued.', async () => {
      await forwardMutation.mutateAsync(buildTelegramForwardRequest(telegramMessage.value, providerChatId))
      forwardTargetChatId.value = ''
    })
  }

  function deleteMessage(): Promise<void> {
    if (!window.confirm('Delete this Telegram message from the provider?')) return Promise.resolve()
    return run('Telegram delete command queued.', () => deleteMutation.mutateAsync(
      buildTelegramDeleteRequest(telegramMessage.value, commandId())
    ))
  }

  function reactionRequest() {
    return buildTelegramReactionRequest(telegramMessage.value, reactionEmoji.value)
  }

  function addReaction(): Promise<void> {
    const request = reactionRequest()
    if (!request.reaction_emoji) return Promise.resolve()
    return run('Telegram reaction command queued.', () => addReactionMutation.mutateAsync(
      buildTelegramReactionMutationRequest(telegramMessage.value, request.reaction_emoji)
    ))
  }

  function removeReaction(): Promise<void> {
    const request = reactionRequest()
    if (!request.reaction_emoji) return Promise.resolve()
    return run('Telegram reaction removal queued.', () => removeReactionMutation.mutateAsync(
      buildTelegramReactionMutationRequest(telegramMessage.value, request.reaction_emoji)
    ))
  }

  function markReadMessage(): Promise<void> {
    return run('Telegram message marked read.', () => markReadMutation.mutateAsync(
      buildTelegramMarkReadRequest(telegramMessage.value)
    ))
  }

  function pinMessage(): Promise<void> {
    return run('Telegram pin command queued.', () => pinMutation.mutateAsync(
      buildTelegramPinRequest(telegramMessage.value)
    ))
  }

  function restoreMessage(): Promise<void> {
    return run('Telegram visibility restore queued.', () => restoreMutation.mutateAsync(
      buildTelegramRestoreRequest(telegramMessage.value, commandId())
    ))
  }

  return {
    editText,
    forwardTargetChatId,
    replyText,
    reactionEmoji,
    status,
    error,
    forwardTargets,
    reactionsQuery,
    versionsQuery,
    tombstonesQuery,
    replyChainQuery,
    forwardChainQuery,
    rawEvidenceQuery,
    isRunning,
    editMessage,
    replyToMessage,
    forwardMessage,
    deleteMessage,
    addReaction,
    removeReaction,
    markReadMessage,
    pinMessage,
    restoreMessage,
  }
}
