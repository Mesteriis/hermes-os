<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../shared/ui/Icon.vue'
import EventTracePanel from './EventTracePanel.vue'
import {
  useEventTraceByCorrelationIdQuery,
  useEventTraceByEventIdQuery
} from './queries'

type TraceLookupMode = 'event' | 'correlation'

const lookupMode = ref<TraceLookupMode>('event')
const inputValue = ref('')
const submittedValue = ref('')

const eventLookupId = computed(() => lookupMode.value === 'event' ? submittedValue.value : null)
const correlationLookupId = computed(() => lookupMode.value === 'correlation' ? submittedValue.value : null)
const eventTraceQuery = useEventTraceByEventIdQuery(eventLookupId)
const correlationTraceQuery = useEventTraceByCorrelationIdQuery(correlationLookupId)

const activeQuery = computed(() => lookupMode.value === 'event' ? eventTraceQuery : correlationTraceQuery)
const trace = computed(() => activeQuery.value.data.value ?? null)
const isLoading = computed(() => activeQuery.value.isFetching.value)
const errorMessage = computed(() => {
  const error = activeQuery.value.error.value
  if (!error) return ''
  return error instanceof Error ? error.message : 'Trace request failed'
})

function submitLookup(): void {
  submittedValue.value = inputValue.value.trim()
}
</script>

<template>
  <section class="event-trace-workspace">
    <header class="workspace-toolbar">
      <div class="toolbar-title">
        <Icon icon="tabler:route" />
        <div>
          <h1>Event Traces</h1>
          <span>event_log</span>
        </div>
      </div>
      <form class="trace-search" @submit.prevent="submitLookup">
        <div class="mode-toggle" aria-label="Trace lookup mode">
          <button
            type="button"
            :class="{ active: lookupMode === 'event' }"
            @click="lookupMode = 'event'"
          >
            Event
          </button>
          <button
            type="button"
            :class="{ active: lookupMode === 'correlation' }"
            @click="lookupMode = 'correlation'"
          >
            Trace
          </button>
        </div>
        <input
          v-model="inputValue"
          spellcheck="false"
          :placeholder="lookupMode === 'event' ? 'event_id' : 'correlation_id'"
        >
        <button type="submit" class="search-button" aria-label="Fetch trace">
          <Icon icon="tabler:search" />
        </button>
      </form>
    </header>

    <EventTracePanel
      :trace="trace"
      :is-loading="isLoading"
      :error-message="errorMessage"
    />
  </section>
</template>

<style scoped>
.event-trace-workspace {
  display: flex;
  height: 100%;
  min-height: 0;
  flex-direction: column;
  background: var(--hh-bg-primary, #fff);
}

.workspace-toolbar,
.toolbar-title,
.trace-search,
.mode-toggle {
  display: flex;
  align-items: center;
}

.workspace-toolbar {
  justify-content: space-between;
  gap: 1rem;
  padding: 0.875rem 1rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.toolbar-title {
  min-width: 0;
  gap: 0.625rem;
}

.toolbar-title svg {
  width: 1.25rem;
  height: 1.25rem;
  color: var(--hh-accent, #2563eb);
}

.toolbar-title h1 {
  margin: 0;
  font-size: 1rem;
  line-height: 1.2;
}

.toolbar-title span {
  font-size: 0.75rem;
  color: var(--hh-text-muted, #6b7280);
}

.trace-search {
  min-width: min(34rem, 58vw);
  gap: 0.5rem;
}

.mode-toggle {
  flex: 0 0 auto;
  overflow: hidden;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
}

.mode-toggle button,
.search-button {
  min-height: 2rem;
  border: 0;
  background: transparent;
  color: var(--hh-text-secondary, #4b5563);
  cursor: pointer;
}

.mode-toggle button {
  padding: 0 0.625rem;
  font-size: 0.8125rem;
}

.mode-toggle button.active {
  background: var(--hh-bg-selected, #eff6ff);
  color: var(--hh-accent, #2563eb);
}

.trace-search input {
  min-width: 0;
  flex: 1;
  height: 2rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  padding: 0 0.625rem;
  background: var(--hh-bg-secondary, #f9fafb);
  color: var(--hh-text-primary, #1f2937);
  font: inherit;
}

.search-button {
  display: inline-flex;
  width: 2rem;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
}

.search-button svg {
  width: 1rem;
  height: 1rem;
}
</style>
