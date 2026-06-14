<script setup lang="ts">
import { useKnowledgeStore, contradictionSeverityTone, formatContradictionClaim, formatContradictionTime, formatContradictionSource } from '../stores/knowledge'
import Icon from '../../../shared/ui/Icon.vue'

const store = useKnowledgeStore()

const props = defineProps<{
  observations: import('../types/knowledge').ContradictionObservation[]
  error: string
  loading: boolean
}>()

function severityClass(severity: import('../types/knowledge').ContradictionSeverity): string {
  return `severity-${severity}`
}

async function handleReview(
  observation: import('../types/knowledge').ContradictionObservation,
  reviewState: Exclude<import('../types/knowledge').ContradictionReviewState, 'suggested'>
) {
  await store.reviewContradictionObservation(observation, reviewState)
}
</script>

<template>
  <div class="polygraph-review">
    <div v-if="error" class="state-line">
      <Icon icon="tabler:alert-circle" class="state-icon error-icon" />
      <span>{{ error }}</span>
    </div>

    <div v-else-if="loading" class="state-line">
      <Icon icon="tabler:loader-2" class="state-icon spin" />
      <span>Loading contradiction observations...</span>
    </div>

    <div v-else-if="observations.length === 0" class="state-line">
      <Icon icon="tabler:check-circle" class="state-icon success-icon" />
      <span>No contradictions detected</span>
    </div>

    <div v-else class="observations-list">
      <div
        v-for="obs in observations"
        :key="obs.observation_id"
        :class="['observation-card', severityClass(obs.severity)]"
      >
        <div class="obs-header">
          <span :class="['severity-badge', 'badge-' + obs.severity]">{{ obs.severity }}</span>
          <span class="obs-time">{{ formatContradictionTime(obs.created_at) }}</span>
        </div>
        <p class="obs-claim">{{ formatContradictionClaim(obs) }}</p>
        <p class="obs-source">
          {{ formatContradictionSource(obs.old_source_kind, obs.old_source_id) }}
        </p>
        <p class="obs-source">
          {{ formatContradictionSource(obs.new_source_kind, obs.new_source_id) }}
        </p>
        <div v-if="obs.review_state === 'suggested'" class="obs-actions">
          <button
            type="button"
            class="action-btn confirm-btn"
            :disabled="store.reviewingContradictionObservationId === obs.observation_id"
            @click="handleReview(obs, 'user_confirmed')"
          >
            <Icon icon="tabler:check" />
            Confirm
          </button>
          <button
            type="button"
            class="action-btn reject-btn"
            :disabled="store.reviewingContradictionObservationId === obs.observation_id"
            @click="handleReview(obs, 'user_rejected')"
          >
            <Icon icon="tabler:x" />
            Reject
          </button>
        </div>
        <div v-else class="obs-reviewed">
          <span class="review-badge">{{ obs.review_state }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.polygraph-review {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.state-line {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  color: hsl(var(--muted-foreground));
  font-size: 13px;
}

.state-icon {
  font-size: 18px;
  flex-shrink: 0;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error-icon { color: hsl(var(--destructive)); }
.success-icon { color: hsl(var(--success)); }

.observations-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.observation-card {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  padding: 12px;
}

.observation-card.severity-critical {
  border-left: 3px solid hsl(var(--destructive));
}

.observation-card.severity-high {
  border-left: 3px solid hsl(var(--warning));
}

.observation-card.severity-medium {
  border-left: 3px solid hsl(var(--primary));
}

.observation-card.severity-low {
  border-left: 3px solid hsl(var(--muted));
}

.obs-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.severity-badge {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  padding: 2px 6px;
  border-radius: 4px;
}

.badge-critical { background: hsl(var(--destructive) / 0.15); color: hsl(var(--destructive)); }
.badge-high { background: hsl(var(--warning) / 0.15); color: hsl(var(--warning)); }
.badge-medium { background: hsl(var(--primary) / 0.15); color: hsl(var(--primary)); }
.badge-low { background: hsl(var(--muted) / 0.3); color: hsl(var(--muted-foreground)); }

.obs-time {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  margin-left: auto;
}

.obs-claim {
  font-size: 13px;
  color: hsl(var(--foreground));
  margin: 0 0 6px;
  line-height: 1.4;
}

.obs-source {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  margin: 0 0 2px;
}

.obs-actions {
  display: flex;
  gap: 8px;
  margin-top: 10px;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  font-size: 12px;
  border-radius: 6px;
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  cursor: pointer;
  transition: background 0.15s;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.confirm-btn:hover:not(:disabled) {
  background: hsl(var(--success) / 0.1);
  border-color: hsl(var(--success));
}

.reject-btn:hover:not(:disabled) {
  background: hsl(var(--destructive) / 0.1);
  border-color: hsl(var(--destructive));
}

.obs-reviewed {
  margin-top: 8px;
}

.review-badge {
  font-size: 11px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: 4px;
  background: hsl(var(--muted) / 0.3);
  color: hsl(var(--muted-foreground));
}
</style>
