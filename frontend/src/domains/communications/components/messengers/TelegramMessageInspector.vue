<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Button, ButtonGroup, Icon } from '@/shared/ui'
import type { TelegramMessage } from '@/shared/communications/types/telegram'
import { telegramForwardTargets } from '../../queries/telegramWorkspacePresentation'
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
} from '../../queries/telegramBusinessQueries'

const props = defineProps<{
  telegramMessage: TelegramMessage
}>()

const editText = ref(props.telegramMessage.text)
const forwardTargetChatId = ref('')
const replyText = ref('')
const reactionEmoji = ref('👍')
const status = ref('')
const error = ref('')
const messageId = computed(() => props.telegramMessage.message_id)
const accountId = computed(() => props.telegramMessage.account_id)
const forwardTargetsQuery = useTelegramChatsQuery(accountId, 200)
const forwardTargets = computed(() =>
  telegramForwardTargets(forwardTargetsQuery.data.value ?? [], props.telegramMessage.provider_chat_id)
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
  editText.value = props.telegramMessage.text
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
  return typeof crypto?.randomUUID === 'function'
    ? crypto.randomUUID()
    : `telegram-${Date.now()}-${Math.random().toString(36).slice(2)}`
}

function editMessage(): Promise<void> {
  const newText = editText.value.trim()
  if (!newText || newText === props.telegramMessage.text.trim()) return Promise.resolve()
  return run('Telegram edit command queued.', () => editMutation.mutateAsync({
    message_id: props.telegramMessage.message_id,
    command_id: commandId(),
    account_id: props.telegramMessage.account_id,
    provider_chat_id: props.telegramMessage.provider_chat_id ?? '',
    provider_message_id: props.telegramMessage.provider_message_id,
    new_text: newText,
  }))
}

function replyToMessage(): Promise<void> {
  const text = replyText.value.trim()
  if (!text) return Promise.resolve()
  return run('Telegram reply command queued.', async () => {
    await replyMutation.mutateAsync({ message_id: props.telegramMessage.message_id, text })
    replyText.value = ''
  })
}

function forwardMessage(): Promise<void> {
  const providerChatId = forwardTargetChatId.value.trim()
  if (!providerChatId) return Promise.resolve()
  return run('Telegram forward command queued.', async () => {
    await forwardMutation.mutateAsync({
      message_id: props.telegramMessage.message_id,
      provider_chat_id: providerChatId,
    })
    forwardTargetChatId.value = ''
  })
}

function deleteMessage(): Promise<void> {
  if (!window.confirm('Delete this Telegram message from the provider?')) return Promise.resolve()
  return run('Telegram delete command queued.', () => deleteMutation.mutateAsync({
    message_id: props.telegramMessage.message_id,
    command_id: commandId(),
    account_id: props.telegramMessage.account_id,
    provider_chat_id: props.telegramMessage.provider_chat_id ?? '',
    provider_message_id: props.telegramMessage.provider_message_id,
    reason_class: 'deleted_by_owner',
    actor_class: 'owner',
    is_provider_delete: true,
  }))
}

function reactionRequest() {
  return {
    account_id: props.telegramMessage.account_id,
    provider_chat_id: props.telegramMessage.provider_chat_id ?? '',
    provider_message_id: props.telegramMessage.provider_message_id,
    reaction_emoji: reactionEmoji.value.trim(),
  }
}

function addReaction(): Promise<void> {
  const request = reactionRequest()
  if (!request.reaction_emoji) return Promise.resolve()
  return run('Telegram reaction command queued.', () => addReactionMutation.mutateAsync({
    messageId: props.telegramMessage.message_id,
    request,
  }))
}

function removeReaction(): Promise<void> {
  const request = reactionRequest()
  if (!request.reaction_emoji) return Promise.resolve()
  return run('Telegram reaction removal queued.', () => removeReactionMutation.mutateAsync({
    messageId: props.telegramMessage.message_id,
    request,
  }))
}
</script>

<template>
  <section class="telegram-message-inspector" aria-label="Telegram message evidence">
    <header class="telegram-message-inspector__header">
      <Icon icon="tabler:message-circle" size="1rem" />
      <div>
        <strong>Message evidence</strong>
        <span>{{ telegramMessage.provider_message_id }}</span>
      </div>
    </header>

    <ButtonGroup aria-label="Telegram message commands" class="telegram-message-inspector__commands">
      <Button size="sm" variant="outline" icon="tabler:mail-opened" :disabled="isRunning" @click="run('Telegram message marked read.', () => markReadMutation.mutateAsync({ message_id: telegramMessage.message_id, account_id: telegramMessage.account_id, provider_chat_id: telegramMessage.provider_chat_id ?? '' }))">
        Mark read
      </Button>
      <Button size="sm" variant="outline" icon="tabler:pin" :disabled="isRunning" @click="run('Telegram message pin command queued.', () => pinMutation.mutateAsync({ message_id: telegramMessage.message_id }))">
        Pin
      </Button>
      <Button size="sm" variant="outline" icon="tabler:trash" :disabled="isRunning" @click="deleteMessage">
        Delete
      </Button>
      <Button size="sm" variant="outline" icon="tabler:eye" :disabled="isRunning" @click="run('Telegram visibility restore queued.', () => restoreMutation.mutateAsync({ message_id: telegramMessage.message_id, command_id: commandId(), account_id: telegramMessage.account_id, provider_chat_id: telegramMessage.provider_chat_id ?? '', provider_message_id: telegramMessage.provider_message_id, reason: 'owner_requested_restore' }))">
        Restore
      </Button>
    </ButtonGroup>

    <p v-if="status" class="telegram-message-inspector__status" role="status">{{ status }}</p>
    <p v-if="error" class="telegram-message-inspector__status telegram-message-inspector__status--error" role="alert">{{ error }}</p>

    <label class="telegram-message-inspector__field">
      <span>Edit projected text</span>
      <textarea v-model="editText" :disabled="isRunning" rows="3" />
    </label>
    <Button size="sm" variant="outline" icon="tabler:pencil" :disabled="isRunning" @click="editMessage">Save edit</Button>

    <label class="telegram-message-inspector__field">
      <span>Reply</span>
      <textarea v-model="replyText" :disabled="isRunning" rows="2" />
    </label>
    <Button size="sm" variant="outline" icon="tabler:corner-up-left" :disabled="isRunning" @click="replyToMessage">Send reply</Button>

    <label class="telegram-message-inspector__field">
      <span>Forward to</span>
      <select v-model="forwardTargetChatId" :disabled="isRunning || !forwardTargets.length">
        <option value="" disabled>Select a Telegram chat</option>
        <option v-for="chat in forwardTargets" :key="chat.telegram_chat_id" :value="chat.provider_chat_id">
          {{ chat.title }}{{ chat.username ? ` (@${chat.username})` : '' }}
        </option>
      </select>
    </label>
    <Button size="sm" variant="outline" icon="tabler:forward" :disabled="isRunning || !forwardTargetChatId" @click="forwardMessage">Forward</Button>

    <section class="telegram-message-inspector__section">
      <h3>Reactions</h3>
      <input v-model="reactionEmoji" :disabled="isRunning" type="text" aria-label="Reaction emoji" />
      <ButtonGroup aria-label="Telegram reaction commands">
        <Button size="sm" variant="outline" :disabled="isRunning" @click="addReaction">React</Button>
        <Button size="sm" variant="outline" :disabled="isRunning" @click="removeReaction">Remove reaction</Button>
      </ButtonGroup>
      <p v-if="!reactionsQuery.data.value?.reactions.length">No projected reactions.</p>
      <p v-for="reaction in reactionsQuery.data.value?.reactions ?? []" :key="reaction.reaction_id">
        {{ reaction.reaction_emoji }} {{ reaction.sender_display_name ?? reaction.sender_id }}
      </p>
    </section>

    <section class="telegram-message-inspector__section">
      <h3>Versions</h3>
      <p v-if="!versionsQuery.data.value?.versions.length">No recorded edits.</p>
      <p v-for="version in versionsQuery.data.value?.versions ?? []" :key="version.version_id">
        v{{ version.version_number }} · {{ version.edit_timestamp }}
      </p>
    </section>

    <section class="telegram-message-inspector__section">
      <h3>Lifecycle evidence</h3>
      <p v-if="!tombstonesQuery.data.value?.tombstones.length">No tombstones.</p>
      <p v-for="tombstone in tombstonesQuery.data.value?.tombstones ?? []" :key="tombstone.tombstone_id">
        {{ tombstone.reason_class }} · {{ tombstone.observed_at }}
      </p>
      <p v-for="reply in replyChainQuery.data.value?.reply_to ?? []" :key="reply.reply_ref_id">
        Reply to {{ reply.target_message_summary?.provider_message_id ?? 'projected message' }}
      </p>
      <p v-for="forward in forwardChainQuery.data.value?.forwards ?? []" :key="forward.forward_ref_id">
        Forwarded from {{ forward.forward_origin_chat_id ?? 'unknown chat' }}
      </p>
      <p v-if="rawEvidenceQuery.data.value?.raw_record">Raw record {{ rawEvidenceQuery.data.value.raw_record.raw_record_id }}</p>
    </section>
  </section>
</template>
