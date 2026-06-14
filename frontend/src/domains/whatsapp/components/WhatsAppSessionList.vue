<script setup lang="ts">
import { ref, computed } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { WhatsappWebSession } from '../types/whatsapp'

const props = defineProps<{
  whatsappSessions: WhatsappWebSession[]
  selectedWhatsappSessionId: string
  isWhatsappLoading: boolean
}>()

const emit = defineEmits<{
  selectSession: [session: WhatsappWebSession]
}>()

const { t } = useI18n()

const searchQuery = ref('')

const filteredSessions = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return props.whatsappSessions
  return props.whatsappSessions.filter((s) => {
    const searchable = [s.device_name, s.account_id, s.session_id, s.link_state]
      .join(' ')
      .toLowerCase()
    return searchable.includes(q)
  })
})

const parentRef = ref<HTMLDivElement | null>(null)

const virtualOptions = computed(() => ({
  count: filteredSessions.value.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 60,
  overscan: 5
}))

const virtualizer = useVirtualizer(virtualOptions)

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())

function sessionLinkStateClass(state: string): string {
  if (state === 'linked' || state === 'fixture') return 'link-state-ok'
  if (state === 'degraded') return 'link-state-warn'
  return 'link-state-err'
}
</script>

<template>
  <section class="panel conversation-list">
    <label class="local-search">
      <Icon icon="tabler:search" width="17" height="17" />
      <input
        v-model="searchQuery"
        :placeholder="t('Search WhatsApp sessions...')"
        autocomplete="off"
      />
    </label>

    <div ref="parentRef" class="session-list-scroll">
      <div v-if="isWhatsappLoading && whatsappSessions.length === 0" class="empty-panel">
        {{ t('Loading WhatsApp Web state...') }}
      </div>
      <div v-else-if="whatsappSessions.length === 0" class="empty-panel">
        {{ t('No WhatsApp Web sessions saved yet.') }}
      </div>
      <template v-else>
        <div :style="{ height: `${totalSize}px` }">
          <button
            v-for="vitem in virtualItems"
            :key="filteredSessions[vitem.index].session_id"
            type="button"
            :class="['session-row', { active: selectedWhatsappSessionId === filteredSessions[vitem.index].session_id }]"
            :style="{ transform: `translateY(${vitem.start}px)`, height: `${vitem.size}px` }"
            @click="emit('selectSession', filteredSessions[vitem.index])"
          >
            <span class="round-icon cyan">
              <Icon icon="tabler:brand-whatsapp" width="22" height="22" />
            </span>
            <div class="session-info">
              <div class="session-title">{{ filteredSessions[vitem.index].device_name }}</div>
              <div class="session-meta">
                <span :class="sessionLinkStateClass(filteredSessions[vitem.index].link_state)">{{ filteredSessions[vitem.index].link_state }}</span>
                <span>{{ filteredSessions[vitem.index].companion_runtime }}</span>
              </div>
            </div>
          </button>
        </div>
      </template>
    </div>
  </section>
</template>

<style scoped>
.session-list-scroll {
  flex: 1;
  overflow-y: auto;
}

.session-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.6rem 0.75rem;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
  transition: background 0.15s ease;
}
.session-row:hover {
  background: var(--bg-hover);
}
.session-row.active {
  background: var(--bg-active, var(--accent-bg));
  font-weight: 500;
}
.session-info {
  flex: 1;
  min-width: 0;
}
.session-title {
  font-size: 0.875rem;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.session-meta {
  display: flex;
  gap: 0.4rem;
  font-size: 0.75rem;
  color: var(--text-secondary);
}
.link-state-ok { color: var(--success, #22c55e); }
.link-state-warn { color: var(--warning, #eab308); }
.link-state-err { color: var(--danger, #ef4444); }
</style>
