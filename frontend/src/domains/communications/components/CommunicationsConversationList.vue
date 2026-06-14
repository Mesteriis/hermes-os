<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import type { CommunicationMessageSummary, NavigatorMode } from '../types/communications'
import { senderLabel, messageTime } from '../stores/communications'

const props = defineProps<{
  messages: CommunicationMessageSummary[]
  selectedIndex: number
  navigatorMode: NavigatorMode
}>()

const emit = defineEmits<{
  select: [index: number]
  'update:navigatorMode': [mode: NavigatorMode]
}>()

// Group messages by sender for contacts mode
type ContactGroup = {
  email: string
  label: string
  messages: CommunicationMessageSummary[]
  latestTime: string | null
  unreadCount: number
}

const contactGroups = computed<ContactGroup[]>(() => {
  const groups = new Map<string, CommunicationMessageSummary[]>()
  for (const msg of props.messages) {
    const emailMatch = msg.sender.match(/<(.+?)>/)
    const email = emailMatch ? emailMatch[1] : msg.sender
    if (!groups.has(email)) groups.set(email, [])
    groups.get(email)!.push(msg)
  }
  return Array.from(groups.entries())
    .map(([email, msgs]) => {
      const latest = msgs.reduce((a, b) =>
        (b.projected_at > a.projected_at) ? b : a
      )
      return {
        email,
        label: senderLabel(msgs[0].sender),
        messages: msgs,
        latestTime: latest.projected_at ?? latest.occurred_at,
        unreadCount: msgs.filter(m => m.workflow_state === 'new').length
      }
    })
    .sort((a, b) => {
      const aTime = a.latestTime ?? ''
      const bTime = b.latestTime ?? ''
      return bTime.localeCompare(aTime)
    })
})

function getMessageIndex(msg: CommunicationMessageSummary): number {
  return props.messages.indexOf(msg)
}
</script>

<template>
  <div class="conversation-list">
    <!-- Navigator mode toggle -->
    <div class="nav-mode-toggle">
      <button
        class="mode-btn"
        :class="{ active: navigatorMode === 'threads' }"
        @click="emit('update:navigatorMode', 'threads')"
      >
        <Icon icon="tabler:list" /> Threads
      </button>
      <button
        class="mode-btn"
        :class="{ active: navigatorMode === 'contacts' }"
        @click="emit('update:navigatorMode', 'contacts')"
      >
        <Icon icon="tabler:users" /> Contacts
      </button>
    </div>

    <!-- Threads mode -->
    <div v-if="navigatorMode === 'threads'" class="thread-list">
      <div
        v-for="(msg, i) in messages"
        :key="msg.message_id"
        class="thread-item"
        :class="{ selected: i === selectedIndex }"
        @click="emit('select', i)"
      >
        <div class="thread-sender">{{ senderLabel(msg.sender) }}</div>
        <div class="thread-subject-row">
          <span v-if="msg.workflow_state === 'new'" class="unread-dot" />
          <span class="thread-subject">{{ msg.subject }}</span>
        </div>
        <div class="thread-time">{{ messageTime(msg.projected_at ?? msg.occurred_at) }}</div>
      </div>
    </div>

    <!-- Contacts mode -->
    <div v-else class="contact-list">
      <div v-for="group in contactGroups" :key="group.email" class="contact-group">
        <div class="contact-header">
          <span class="contact-label">{{ group.label }}</span>
          <span class="contact-count">{{ group.messages.length }}</span>
        </div>
        <div
          v-for="msg in group.messages"
          :key="msg.message_id"
          class="contact-message"
          :class="{ selected: getMessageIndex(msg) === selectedIndex }"
          @click="emit('select', getMessageIndex(msg))"
        >
          <span v-if="msg.workflow_state === 'new'" class="unread-dot" />
          <span class="contact-msg-subject">{{ msg.subject }}</span>
          <span class="contact-msg-time">{{ messageTime(msg.projected_at ?? msg.occurred_at) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.conversation-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.nav-mode-toggle {
  display: flex;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  padding: 0.25rem;
  gap: 0.25rem;
}

.mode-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  padding: 0.375rem 0.5rem;
  border: none;
  background: transparent;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
  cursor: pointer;
  transition: background-color 0.1s;
}
.mode-btn:hover {
  background: var(--hh-bg-hover, #f3f4f6);
}
.mode-btn.active {
  background: var(--hh-bg-selected, #eff6ff);
  color: var(--hh-accent, #3b82f6);
  font-weight: 500;
}

.thread-list,
.contact-list {
  flex: 1;
  overflow-y: auto;
}

.thread-item {
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}
.thread-item:hover {
  background: var(--hh-bg-hover, #f3f4f6);
}
.thread-item.selected {
  background: var(--hh-bg-selected, #eff6ff);
  border-left: 3px solid var(--hh-accent, #3b82f6);
}

.thread-sender {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-primary, #1f2937);
}

.thread-subject-row {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.unread-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--hh-accent, #3b82f6);
  flex-shrink: 0;
}

.thread-subject {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.thread-time {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
  text-align: right;
}

.contact-group {
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.contact-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: var(--hh-bg-secondary, #f9fafb);
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
}

.contact-count {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
}

.contact-message {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem 0.375rem 1.5rem;
  cursor: pointer;
  font-size: 0.75rem;
}
.contact-message:hover {
  background: var(--hh-bg-hover, #f3f4f6);
}
.contact-message.selected {
  background: var(--hh-bg-selected, #eff6ff);
}

.contact-msg-subject {
  flex: 1;
  color: var(--hh-text-primary, #1f2937);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.contact-msg-time {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
  white-space: nowrap;
}
</style>
