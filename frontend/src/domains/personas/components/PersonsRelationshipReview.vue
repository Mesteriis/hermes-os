<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { computed } from 'vue'
import { formatRelationshipType, formatRelationshipScore } from '../stores/personas'
import type { Relationship } from '../types/persona'

const { t } = useI18n()

const props = defineProps<{
  relationships: Relationship[]
  selectedPersonaId: string | null
  isLoading: boolean
  error: string
  reviewingRelationshipId: string | null
  onReload: () => Promise<void>
  onReview: (relationship: Relationship, reviewState: string) => Promise<void>
}>()

const suggestedRelationships = computed(() =>
  props.relationships.filter((r) => r.review_state === 'suggested')
)

function relationshipPeer(relationship: Relationship): string {
  const selId = props.selectedPersonaId
  if (selId && relationship.source_entity_id === selId) {
    return `${relationship.target_entity_kind}:${relationship.target_entity_id.slice(0, 8)}...`
  }
  if (selId && relationship.target_entity_id === selId) {
    return `${relationship.source_entity_kind}:${relationship.source_entity_id.slice(0, 8)}...`
  }
  return `${relationship.source_entity_kind}:${relationship.source_entity_id.slice(0, 8)}... → ${relationship.target_entity_kind}:${relationship.target_entity_id.slice(0, 8)}...`
}
</script>

<template>
  <div class="widget-frame" data-widget-id="persons-relationship-review">
    <section class="panel info-card relationship-review-panel" :aria-busy="isLoading">
      <header>
        <div>
          <span class="panel-kicker">{{ t('Relationships') }}</span>
          <h2>{{ t('Relationship Review') }}</h2>
        </div>
        <button type="button" :title="t('Reload relationships')" @click="() => onReload()" :disabled="isLoading">
          <Icon icon="tabler:refresh" :size="15" />
        </button>
      </header>

      <div v-if="error" class="relationship-review-state error">
        <span>{{ error }}</span>
        <button type="button" @click="() => onReload()" :disabled="isLoading">{{ t('Retry') }}</button>
      </div>
      <div v-else-if="isLoading" class="relationship-review-state">
        <span>{{ t('Loading relationships') }}</span>
      </div>
      <div v-else-if="suggestedRelationships.length === 0" class="relationship-review-state">
        <span>{{ t('No suggested relationships') }}</span>
      </div>
      <div v-else class="relationship-review-list">
        <article v-for="relationship in suggestedRelationships" :key="relationship.relationship_id" class="relationship-review-item">
          <div>
            <strong>{{ formatRelationshipType(relationship.relationship_type) }}</strong>
            <p>{{ relationshipPeer(relationship) }}</p>
            <small>
              {{ t('Trust') }}: {{ formatRelationshipScore(relationship.trust_score) }}
              · {{ t('Strength') }}: {{ formatRelationshipScore(relationship.strength_score) }}
              · {{ t('Confidence') }}: {{ formatRelationshipScore(relationship.confidence) }}
            </small>
          </div>
          <div class="relationship-review-actions">
            <button
              type="button"
              :disabled="reviewingRelationshipId === relationship.relationship_id"
              @click="() => onReview(relationship, 'user_confirmed')"
            >
              <Icon icon="tabler:check" :size="14" /> {{ t('Confirm') }}
            </button>
            <button
              type="button"
              :disabled="reviewingRelationshipId === relationship.relationship_id"
              @click="() => onReview(relationship, 'user_rejected')"
            >
              <Icon icon="tabler:x" :size="14" /> {{ t('Reject') }}
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
.relationship-review-actions {
  display: flex;
  gap: 7px;
  flex-wrap: wrap;
}
</style>
