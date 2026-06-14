<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { computed } from 'vue'
import { formatIdentityTraceKind, formatIdentityTraceValue, identityTraceConfidence } from '../stores/personas'
import type { PersonIdentity, PersonaOption } from '../types/persona'

const { t } = useI18n()

const props = defineProps<{
  identityTraces: PersonIdentity[]
  persons: PersonaOption[]
  selectedPersonaId: string | null
  isLoading: boolean
  error: string
  assigningIdentityTraceId: string | null
  onReload: () => Promise<void>
  onAssign: (trace: PersonIdentity, personId: string) => Promise<void>
}>()

const pendingIdentityTraces = computed(() =>
  props.identityTraces.filter((trace) => trace.person_id === null)
)

const selectedPersonaByTrace: Record<string, string> = {}

function targetPersonaId(trace: PersonIdentity): string {
  return selectedPersonaByTrace[trace.id] ?? props.selectedPersonaId ?? props.persons[0]?.person_id ?? ''
}

function personaLabel(person: PersonaOption): string {
  return person.company ? `${person.name} · ${person.company}` : person.name
}
</script>

<template>
  <div class="widget-frame" data-widget-id="persons-identity-trace-review">
    <section class="panel info-card relationship-review-panel" :aria-busy="isLoading">
      <header>
        <div>
          <span class="panel-kicker">{{ t('Identity Resolution') }}</span>
          <h2>{{ t('Unattached Traces') }}</h2>
        </div>
        <button type="button" :title="t('Reload identity traces')" @click="() => onReload()" :disabled="isLoading">
          <Icon icon="tabler:refresh" :size="15" />
        </button>
      </header>

      <div v-if="error" class="relationship-review-state error">
        <span>{{ error }}</span>
        <button type="button" @click="() => onReload()" :disabled="isLoading">{{ t('Retry') }}</button>
      </div>
      <div v-else-if="isLoading" class="relationship-review-state">
        <span>{{ t('Loading identity traces') }}</span>
      </div>
      <div v-else-if="pendingIdentityTraces.length === 0" class="relationship-review-state">
        <span>{{ t('No unattached identity traces') }}</span>
      </div>
      <div v-else class="relationship-review-list">
        <article v-for="trace in pendingIdentityTraces" :key="trace.id" class="relationship-review-item">
          <div>
            <strong>{{ formatIdentityTraceKind(trace.identity_type) }}</strong>
            <p>{{ formatIdentityTraceValue(trace) }}</p>
            <small>
              {{ t('Source') }}: {{ trace.source }}
              · {{ t('Confidence') }}: {{ identityTraceConfidence(trace) }}
            </small>
          </div>
          <div class="identity-trace-target">
            <select
              :value="targetPersonaId(trace)"
              :aria-label="t('Target Persona')"
              @change="(e) => { selectedPersonaByTrace[trace.id] = (e.target as HTMLSelectElement).value }"
            >
              <option v-for="person in persons" :key="person.person_id" :value="person.person_id">
                {{ personaLabel(person) }}
              </option>
            </select>
            <button
              type="button"
              :disabled="assigningIdentityTraceId === trace.id || persons.length === 0"
              @click="() => onAssign(trace, targetPersonaId(trace))"
            >
              <Icon icon="tabler:link" :size="14" /> {{ t('Assign') }}
            </button>
          </div>
        </article>
      </div>
    </section>
  </div>
</template>

<style scoped>
.relationship-review-panel {
  display: grid;
  gap: 10px;
}
.relationship-review-panel header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.relationship-review-panel header button {
  width: 32px;
  padding: 0;
}
.relationship-review-state {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  min-height: 42px;
  color: var(--hh-color-text-muted);
  font-size: 12px;
}
.relationship-review-state.error {
  color: var(--hh-color-danger);
}
.relationship-review-list {
  display: grid;
  gap: 9px;
}
.relationship-review-item {
  display: grid;
  gap: 8px;
  border-top: 1px solid rgba(102, 189, 180, 0.08);
  padding-top: 10px;
}
.relationship-review-item:first-child {
  border-top: none;
  padding-top: 0;
}
.relationship-review-item strong {
  display: block;
  margin-bottom: 3px;
  overflow-wrap: anywhere;
}
.relationship-review-item p,
.relationship-review-item small {
  display: block;
  margin: 0 0 4px;
  color: var(--hh-color-text-muted);
  font-size: 11px;
  line-height: 1.35;
  overflow-wrap: anywhere;
}
.identity-trace-target {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px;
  align-items: center;
}
.identity-trace-target select {
  min-width: 0;
  height: 30px;
  border: 1px solid var(--hh-border-subtle);
  border-radius: var(--hh-radius-control);
  background: rgba(4, 21, 24, 0.72);
  color: var(--hh-color-text-soft);
  padding: 0 8px;
  font-size: 11px;
}
.identity-trace-target button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-height: 30px;
  border: 1px solid var(--hh-border-subtle);
  border-radius: var(--hh-radius-control);
  background: rgba(4, 21, 24, 0.72);
  color: var(--hh-color-text-soft);
  padding: 0 10px;
  font-size: 11px;
}
</style>
