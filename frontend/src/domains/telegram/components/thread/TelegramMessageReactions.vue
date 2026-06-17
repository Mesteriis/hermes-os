<script setup lang="ts">
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import type { TelegramCapabilitiesResponse, TelegramMessage, TelegramOperationCapability } from '../../types/telegram'

const { t } = useI18n()

const props = defineProps<{
  message: TelegramMessage
  capabilities?: TelegramCapabilitiesResponse | null
  isTelegramActionSubmitting: boolean
  isPickerOpen: boolean
}>()

const emit = defineEmits<{
  togglePicker: [messageId: string]
  addReaction: [payload: { message: TelegramMessage; emoji: string }]
  removeReaction: [payload: { message: TelegramMessage; emoji: string }]
}>()

const reactionPalette = ['👍', '👎', '❤️', '🔥', '🥰', '👏', '😁', '🤔', '🤯', '😱', '🤬', '😢', '🎉', '🤩', '🤮', '💩']

type ReactionGroup = {
  reaction_emoji: string
  count: number
  senders: string[]
}

function capability(operation: string): TelegramOperationCapability | undefined {
  return props.capabilities?.capabilities.find((item) => item.operation === operation)
}

function messageReactions(message: TelegramMessage): ReactionGroup[] {
  const summary = message.metadata?.reaction_summary as
    | { reactions?: unknown }
    | undefined
  const reactionItems = summary?.reactions
  if (!Array.isArray(reactionItems)) {
    return []
  }
  return reactionItems
    .filter((item: unknown): item is ReactionGroup => {
      return (
        item !== null &&
        typeof item === 'object' &&
        'reaction_emoji' in item &&
        typeof item.reaction_emoji === 'string' &&
        'count' in item &&
        typeof item.count === 'number' &&
        'senders' in item &&
        Array.isArray(item.senders)
      )
    })
    .map((item) => ({
      reaction_emoji: item.reaction_emoji,
      count: item.count,
      senders: item.senders.filter((sender: unknown): sender is string => typeof sender === 'string'),
    }))
}

function isCapabilityVisible(operation: string): boolean {
  return capability(operation)?.status !== 'unsupported'
}

function capabilityTitle(operation: string, fallbackLabel: string): string {
  const op = capability(operation)
  if (!op) return fallbackLabel
  return op.status === 'available' ? fallbackLabel : `${fallbackLabel}: ${op.reason}`
}

function canReact(): boolean {
  const status = capability('reactions.add')?.status
  return status === 'available' || status === 'degraded'
}

function canRemoveReaction(): boolean {
  const status = capability('reactions.remove')?.status
  return status === 'available' || status === 'degraded'
}

function canRemoveReactionGroup(group: { senders: string[] }): boolean {
  return canRemoveReaction() && group.senders.includes('Owner')
}

function emitReaction(emoji: string) {
  emit('addReaction', { message: props.message, emoji })
}
</script>

<template>
  <div v-if="messageReactions(message).length" class="telegram-reaction-bar">
    <span
      v-for="group in messageReactions(message)"
      :key="group.reaction_emoji"
      class="telegram-reaction-chip"
      :title="group.senders.join(', ')"
    >
      {{ group.reaction_emoji }} {{ group.count }}
      <button
        v-if="canRemoveReactionGroup(group)"
        type="button"
        class="telegram-reaction-remove"
        :title="capabilityTitle('reactions.remove', t('Remove your reaction'))"
        :disabled="isTelegramActionSubmitting"
        @click.stop="emit('removeReaction', { message, emoji: group.reaction_emoji })"
      >
        <Icon icon="tabler:x" width="10" height="10" />
      </button>
    </span>
  </div>
  <div v-if="isCapabilityVisible('reactions.add')" class="telegram-reaction-picker">
    <button
      type="button"
      class="telegram-reaction-trigger"
      :title="capabilityTitle('reactions.add', t('Add reaction'))"
      :disabled="!canReact() || isTelegramActionSubmitting"
      @click.stop="emit('togglePicker', message.message_id)"
    >
      <Icon icon="tabler:mood-smile" width="14" height="14" />
    </button>
    <div
      v-if="isPickerOpen && canReact()"
      class="telegram-emoji-palette"
    >
      <button
        v-for="emoji in reactionPalette"
        :key="emoji"
        type="button"
        class="telegram-emoji-btn"
        :disabled="isTelegramActionSubmitting"
        @click.stop="emitReaction(emoji)"
      >
        {{ emoji }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.telegram-reaction-bar {
  display: flex; flex-wrap: wrap; gap: 2px; margin-top: 4px;
}
.telegram-reaction-chip {
  display: inline-flex; align-items: center; gap: 2px;
  padding: 1px 6px; border-radius: 10px; font-size: 11px;
  background: var(--color-surface-hover, #f0f0f0);
  border: 1px solid var(--color-border, #e0e0e0); cursor: default;
}
.telegram-reaction-remove {
  display: inline-flex; align-items: center; justify-content: center;
  width: 14px; height: 14px; padding: 0; border: none; border-radius: 999px;
  background: transparent; color: var(--color-text-secondary, #777); cursor: pointer; line-height: 1;
}
.telegram-reaction-remove:hover { background: var(--color-danger-subtle, #fde8e8); color: var(--color-danger, #c62828); }
.telegram-reaction-picker { position: relative; display: inline-block; margin-top: 4px; }
.telegram-reaction-trigger {
  display: inline-flex; align-items: center; justify-content: center;
  width: 24px; height: 24px; border: none; background: transparent;
  border-radius: 4px; cursor: pointer; color: var(--color-text-secondary, #777);
}
.telegram-reaction-trigger:hover { background: var(--color-surface-hover, #f0f0f0); }
.telegram-emoji-palette {
  position: absolute; bottom: 100%; left: 0;
  display: flex; flex-wrap: wrap; gap: 2px; padding: 4px;
  background: var(--color-surface, #fff); border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.12);
  z-index: 10; max-width: 200px;
}
.telegram-emoji-btn {
  width: 28px; height: 28px; border: none; background: transparent;
  border-radius: 4px; cursor: pointer; font-size: 16px;
  display: flex; align-items: center; justify-content: center;
}
.telegram-emoji-btn:hover { background: var(--color-primary-subtle, #e3f2fd); }
</style>
