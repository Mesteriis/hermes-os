<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import TelegramCallTranscriptPanel from './TelegramCallTranscriptPanel.vue'
import { useTelegramCallsQuery } from '../queries/useTelegramQuery'

const { t } = useI18n()

const props = defineProps<{
  selectedAccountId: string | null
}>()

const searchQuery = ref('')
const recentCallsQuery = useTelegramCallsQuery(computed(() => props.selectedAccountId ?? undefined), 10)
const recentCalls = computed(() => recentCallsQuery.data.value ?? [])
const filteredCalls = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return recentCalls.value
  return recentCalls.value.filter((call) =>
    [call.call_id, call.provider_chat_id, call.status]
      .join(' ')
      .toLowerCase()
      .includes(query)
  )
})
</script>

<template>
  <article class="telegram-rail-card telegram-calls-panel">
    <header class="telegram-calls-panel__header">
      <div>
        <h3>{{ t('Recent Calls') }}</h3>
        <p>{{ t('Projected Telegram calls for the selected account.') }}</p>
      </div>
      <span class="telegram-calls-panel__count">{{ filteredCalls.length }}</span>
    </header>

    <label v-if="recentCalls.length > 0" class="telegram-calls-panel__search">
      <Icon icon="tabler:search" width="15" height="15" />
      <input
        v-model="searchQuery"
        type="search"
        :placeholder="t('Search projected calls')"
      />
    </label>

    <div v-if="!selectedAccountId" class="telegram-call-placeholder">
      {{ t('Select a Telegram account to inspect projected calls.') }}
    </div>
    <div v-else-if="recentCallsQuery.isLoading.value" class="telegram-call-placeholder">
      {{ t('Loading Telegram calls...') }}
    </div>
    <div v-else-if="recentCalls.length === 0" class="telegram-call-placeholder">
      {{ t('No Telegram calls projected for this account yet.') }}
    </div>
    <div v-else-if="filteredCalls.length === 0" class="telegram-call-placeholder">
      {{ t('No projected calls match this search.') }}
    </div>
    <TelegramCallTranscriptPanel v-else :calls="filteredCalls" />
  </article>
</template>

<style scoped>
.telegram-calls-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.telegram-calls-panel__header,
.telegram-calls-panel__search {
  display: flex;
  align-items: center;
  gap: 8px;
}

.telegram-calls-panel__header {
  justify-content: space-between;
  align-items: flex-start;
}

.telegram-calls-panel__header h3,
.telegram-calls-panel__header p {
  margin: 0;
}

.telegram-calls-panel__header p,
.telegram-calls-panel__search {
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}

.telegram-calls-panel__count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  min-height: 24px;
  padding: 0 8px;
  border-radius: 999px;
  background: var(--color-primary-subtle, #e3f2fd);
  color: var(--color-primary, #0066cc);
  font-size: 11px;
  font-weight: 600;
}

.telegram-calls-panel__search {
  padding: 8px 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 999px;
  background: var(--color-surface, #fff);
}

.telegram-calls-panel__search input {
  flex: 1;
  border: none;
  background: transparent;
  font: inherit;
  color: var(--color-text, #333);
  outline: none;
}
</style>
