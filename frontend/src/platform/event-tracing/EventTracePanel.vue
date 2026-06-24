<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Icon from '../../shared/ui/Icon.vue'
import type { EventTrace, StoredEventEnvelope } from './types'

const props = defineProps<{
  trace: EventTrace | null
  isLoading?: boolean
  errorMessage?: string
}>()

const selectedEventId = ref<string | null>(null)

const orderedEvents = computed(() => props.trace?.events ?? [])
const selectedEvent = computed(() => {
  return orderedEvents.value.find((item) => item.event.event_id === selectedEventId.value) ?? orderedEvents.value[0] ?? null
})
const edgeByChild = computed(() => new Map((props.trace?.edges ?? []).map((edge) => [edge.child_event_id, edge.parent_event_id])))
const annotationsByEvent = computed(() => {
  const map = new Map<string, string[]>()
  for (const annotation of props.trace?.consumer_annotations ?? []) {
    const entries = map.get(annotation.event_id) ?? []
    entries.push(`${annotation.consumer_name}: ${annotation.status}`)
    map.set(annotation.event_id, entries)
  }
  for (const deadLetter of props.trace?.dead_letters ?? []) {
    const entries = map.get(deadLetter.event_id) ?? []
    entries.push(`DLQ: ${deadLetter.consumer_name ?? 'unknown'}`)
    map.set(deadLetter.event_id, entries)
  }
  return map
})

watch(
  orderedEvents,
  (events) => {
    if (!events.some((item) => item.event.event_id === selectedEventId.value)) {
      selectedEventId.value = events[0]?.event.event_id ?? null
    }
  },
  { immediate: true }
)

function shortId(value: string | null | undefined): string {
  if (!value) return 'none'
  if (value.length <= 24) return value
  return `${value.slice(0, 10)}...${value.slice(-8)}`
}

function sourceLabel(event: StoredEventEnvelope): string {
  const source = event.event.source
  const kind = typeof source.kind === 'string' ? source.kind : 'source'
  const account = typeof source.account_id === 'string' ? source.account_id : null
  return account ? `${kind} / ${shortId(account)}` : kind
}

function subjectLabel(event: StoredEventEnvelope): string {
  const subject = event.event.subject
  const kind = typeof subject.kind === 'string' ? subject.kind : 'subject'
  const id = firstString(subject.entity_id, subject.message_id, subject.observation_id, subject.id)
  return id ? `${kind} / ${shortId(id)}` : kind
}

function firstString(...values: unknown[]): string | null {
  for (const value of values) {
    if (typeof value === 'string' && value.trim().length > 0) return value
  }
  return null
}

function jsonPreview(value: unknown): string {
  return JSON.stringify(value, null, 2)
}
</script>

<template>
  <section class="event-trace-panel">
    <div class="trace-header">
      <div class="trace-identity">
        <Icon icon="tabler:route" class="trace-icon" />
        <div>
          <span class="trace-label">Trace</span>
          <strong>{{ shortId(trace?.correlation_id) }}</strong>
        </div>
      </div>
      <div class="trace-metrics" aria-label="Trace metrics">
        <span><strong>{{ trace?.events.length ?? 0 }}</strong> events</span>
        <span><strong>{{ trace?.edges.length ?? 0 }}</strong> edges</span>
        <span><strong>{{ trace?.missing_parent_ids.length ?? 0 }}</strong> missing</span>
        <span><strong>{{ trace?.dead_letters.length ?? 0 }}</strong> DLQ</span>
      </div>
    </div>

    <div v-if="isLoading" class="trace-state">Loading trace</div>
    <div v-else-if="errorMessage" class="trace-state trace-error">{{ errorMessage }}</div>
    <div v-else-if="!trace" class="trace-state">No trace selected</div>
    <div v-else-if="orderedEvents.length === 0" class="trace-state">Trace is empty</div>
    <div v-else class="trace-body">
      <ol class="trace-events">
        <li v-for="item in orderedEvents" :key="item.event.event_id">
          <button
            class="trace-event"
            :class="{ active: selectedEvent?.event.event_id === item.event.event_id }"
            @click="selectedEventId = item.event.event_id"
          >
            <span class="event-position">#{{ item.position }}</span>
            <span class="event-copy">
              <span class="event-type">{{ item.event.event_type }}</span>
              <span class="event-meta">
                {{ shortId(item.event.event_id) }}
                <template v-if="edgeByChild.has(item.event.event_id)">
                  parent {{ shortId(edgeByChild.get(item.event.event_id)) }}
                </template>
                <template v-else-if="trace.root_event_ids.includes(item.event.event_id)">
                  root
                </template>
              </span>
            </span>
            <span v-if="annotationsByEvent.has(item.event.event_id)" class="event-badge">
              {{ annotationsByEvent.get(item.event.event_id)?.length }}
            </span>
          </button>
        </li>
      </ol>

      <aside v-if="selectedEvent" class="trace-detail">
        <div class="detail-title">
          <span>{{ selectedEvent.event.event_type }}</span>
          <code>{{ shortId(selectedEvent.event.event_id) }}</code>
        </div>
        <dl class="detail-grid">
          <div>
            <dt>Source</dt>
            <dd>{{ sourceLabel(selectedEvent) }}</dd>
          </div>
          <div>
            <dt>Subject</dt>
            <dd>{{ subjectLabel(selectedEvent) }}</dd>
          </div>
          <div>
            <dt>Causation</dt>
            <dd>{{ shortId(selectedEvent.event.causation_id) }}</dd>
          </div>
          <div>
            <dt>Recorded</dt>
            <dd>{{ selectedEvent.event.recorded_at }}</dd>
          </div>
        </dl>
        <div v-if="annotationsByEvent.has(selectedEvent.event.event_id)" class="detail-annotations">
          <span v-for="entry in annotationsByEvent.get(selectedEvent.event.event_id)" :key="entry">
            {{ entry }}
          </span>
        </div>
        <pre>{{ jsonPreview(selectedEvent.event.subject) }}</pre>
      </aside>
    </div>
  </section>
</template>

<style scoped>
.event-trace-panel {
  display: flex;
  min-height: 0;
  height: 100%;
  flex-direction: column;
  background: var(--hh-bg-primary, #fff);
  color: var(--hh-text-primary, #1f2937);
}

.trace-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.875rem 1rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.trace-identity,
.trace-metrics,
.trace-event,
.detail-title {
  display: flex;
  align-items: center;
}

.trace-identity {
  min-width: 0;
  gap: 0.625rem;
}

.trace-icon {
  width: 1.25rem;
  height: 1.25rem;
  color: var(--hh-accent, #2563eb);
}

.trace-label {
  display: block;
  font-size: 0.6875rem;
  color: var(--hh-text-muted, #6b7280);
  text-transform: uppercase;
}

.trace-metrics {
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #4b5563);
}

.trace-metrics span,
.event-badge,
.detail-annotations span {
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  padding: 0.1875rem 0.4375rem;
  background: var(--hh-bg-secondary, #f9fafb);
}

.trace-state {
  padding: 2rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.875rem;
}

.trace-error {
  color: var(--hh-danger, #b91c1c);
}

.trace-body {
  display: grid;
  grid-template-columns: minmax(20rem, 0.95fr) minmax(22rem, 1.05fr);
  min-height: 0;
  flex: 1;
}

.trace-events {
  min-height: 0;
  overflow: auto;
  margin: 0;
  padding: 0.75rem;
  list-style: none;
  border-right: 1px solid var(--hh-border, #e5e7eb);
}

.trace-event {
  width: 100%;
  min-height: 3.125rem;
  gap: 0.75rem;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  padding: 0.5rem 0.625rem;
  text-align: left;
  color: inherit;
  cursor: pointer;
}

.trace-event:hover,
.trace-event.active {
  border-color: var(--hh-border-accent-soft, #bfdbfe);
  background: var(--hh-bg-selected, #eff6ff);
}

.event-position,
.event-badge {
  flex: 0 0 auto;
  font-size: 0.75rem;
  color: var(--hh-text-muted, #6b7280);
}

.event-copy {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 0.125rem;
}

.event-type,
.detail-title span {
  overflow-wrap: anywhere;
  font-size: 0.8125rem;
  font-weight: 600;
}

.event-meta,
.detail-grid {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #4b5563);
}

.trace-detail {
  min-width: 0;
  overflow: auto;
  padding: 1rem;
}

.detail-title {
  justify-content: space-between;
  gap: 1rem;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.detail-title code,
.trace-detail pre {
  font-size: 0.75rem;
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
  margin: 0.875rem 0;
}

.detail-grid dt {
  color: var(--hh-text-muted, #6b7280);
}

.detail-grid dd {
  margin: 0.125rem 0 0;
  overflow-wrap: anywhere;
  color: var(--hh-text-primary, #1f2937);
}

.detail-annotations {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  margin-bottom: 0.875rem;
  font-size: 0.75rem;
}

.trace-detail pre {
  overflow: auto;
  max-height: 16rem;
  margin: 0;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  padding: 0.75rem;
  background: var(--hh-bg-secondary, #f9fafb);
  white-space: pre-wrap;
}
</style>
