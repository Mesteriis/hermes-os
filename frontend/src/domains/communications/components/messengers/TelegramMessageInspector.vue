<script setup lang="ts">
import { computed } from 'vue'
import { Button, ButtonGroup, Icon } from '@/shared/ui'
import type { TelegramMessage } from '@/shared/communications/types/telegram'
import { useTelegramMessageInspectorController } from '../../queries/useTelegramMessageInspectorController'

const props = defineProps<{
  telegramMessage: TelegramMessage
}>()

const controller = useTelegramMessageInspectorController(
  computed(() => props.telegramMessage)
)
const {
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
} = controller
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
      <Button size="sm" variant="outline" icon="tabler:mail-opened" :disabled="isRunning" @click="markReadMessage">
        Mark read
      </Button>
      <Button size="sm" variant="outline" icon="tabler:pin" :disabled="isRunning" @click="pinMessage">
        Pin
      </Button>
      <Button size="sm" variant="outline" icon="tabler:trash" :disabled="isRunning" @click="deleteMessage">
        Delete
      </Button>
      <Button size="sm" variant="outline" icon="tabler:eye" :disabled="isRunning" @click="restoreMessage">
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
