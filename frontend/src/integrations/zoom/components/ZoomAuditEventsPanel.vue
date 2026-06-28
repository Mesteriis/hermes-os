<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { ProviderAccount } from '../../../shared/zoom/settingsBridge'
import { useZoomAuditEventsQuery } from '../queries/useZoomRuntimeQuery'

const { t } = useI18n()

const props = defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

const auditEventsQuery = useZoomAuditEventsQuery(
  computed(() => props.selectedAccount?.account_id ?? null),
  12
)
const auditEvents = computed(() => auditEventsQuery.data.value ?? [])

function formatDate(value: string | null | undefined): string {
  if (!value) return '—'
  const parsed = new Date(value)
  if (Number.isNaN(parsed.getTime())) return '—'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(parsed)
}
</script>

<template>
  <section class="integration-section zoom-audit-events">
    <header class="zoom-audit-events__header">
      <div>
        <h4>{{ t('Zoom audit events') }}</h4>
        <p>{{ t('Recent runtime and bridge events for the selected Zoom account.') }}</p>
      </div>
      <span class="zoom-audit-events__count">{{ auditEvents.length }}</span>
    </header>

    <div v-if="!selectedAccount?.account_id" class="zoom-audit-events__placeholder">
      {{ t('Select a Zoom account to inspect recent audit events.') }}
    </div>
    <div v-else-if="auditEventsQuery.isLoading.value" class="zoom-audit-events__placeholder">
      {{ t('Loading Zoom audit events...') }}
    </div>
    <div v-else-if="auditEvents.length === 0" class="zoom-audit-events__placeholder">
      {{ t('No Zoom audit events for this account yet.') }}
    </div>
    <div v-else class="zoom-audit-events__list">
      <article v-for="item in auditEvents" :key="item.event_id" class="zoom-audit-events__item">
        <header>
          <strong>{{ item.event_type }}</strong>
          <small>{{ formatDate(item.occurred_at) }}</small>
        </header>
        <dl class="zoom-audit-events__meta">
          <div><dt>{{ t('Subject') }}</dt><dd>{{ item.subject_kind ?? '—' }}</dd></div>
          <div><dt>{{ t('Entity') }}</dt><dd>{{ item.subject_entity_id ?? '—' }}</dd></div>
          <div><dt>{{ t('Position') }}</dt><dd>{{ item.position }}</dd></div>
          <div><dt>{{ t('Correlation') }}</dt><dd>{{ item.correlation_id ?? '—' }}</dd></div>
        </dl>
      </article>
    </div>
  </section>
</template>

<style scoped>
.zoom-audit-events { display: grid; gap: 12px; }
.zoom-audit-events__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.zoom-audit-events__header h4,
.zoom-audit-events__header p,
.zoom-audit-events__item header {
  margin: 0;
}
.zoom-audit-events__header p,
.zoom-audit-events__meta dt,
.zoom-audit-events__item small {
  color: var(--hh-text-muted);
  font-size: 11px;
}
.zoom-audit-events__count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  min-height: 24px;
  padding: 0 8px;
  border-radius: 999px;
  border: 1px solid var(--hh-border);
  background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%);
  font-size: 11px;
  font-weight: 600;
}
.zoom-audit-events__list,
.zoom-audit-events__item {
  display: grid;
  gap: 8px;
}
.zoom-audit-events__placeholder,
.zoom-audit-events__item {
  padding: 10px 12px;
  border-radius: var(--hh-radius-sm);
  border: 1px solid var(--hh-border);
  background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%);
  font-size: 12px;
}
.zoom-audit-events__item header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}
.zoom-audit-events__item strong {
  display: block;
  font-size: 12px;
}
.zoom-audit-events__meta {
  display: grid;
  gap: 8px;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin: 0;
}
.zoom-audit-events__meta div {
  display: grid;
  gap: 2px;
}
.zoom-audit-events__meta dt,
.zoom-audit-events__meta dd {
  margin: 0;
  word-break: break-word;
}
@media (max-width: 900px) {
  .zoom-audit-events__meta {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
