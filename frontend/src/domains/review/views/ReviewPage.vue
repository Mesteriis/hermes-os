<script setup lang="ts">
import { onMounted } from 'vue'
import { useReviewStore } from '../stores/review'
import Icon from '../../../shared/ui/Icon.vue'

const store = useReviewStore()

onMounted(() => {
  store.loadAll()
})

function relationshipPeer(item: import('../../personas/types/persona').Relationship): string {
  return `${item.source_entity_kind}:${item.source_entity_id} → ${item.target_entity_kind}:${item.target_entity_id}`
}

function decisionEntityLabel(item: import('../../tasks/types/task').Decision): string {
  if (!item.decided_by_entity_kind || !item.decided_by_entity_id) return 'Unknown'
  return `${item.decided_by_entity_kind}:${item.decided_by_entity_id}`
}

function obligationEntityLabel(item: import('../../tasks/types/task').Obligation): string {
  return `${item.obligated_entity_kind}:${item.obligated_entity_id}`
}

function formatItemTime(value: string | null): string {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return ''
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(date)
}

async function handleReview(
  action: import('../types/review').ReviewWorkspaceItemAction
) {
  await store.reviewItem(action)
}
</script>

<template>
  <div class="review-page">
    <!-- Header -->
    <div class="view-header">
      <div class="header-title-group">
        <h2 class="view-title">Review Workspace</h2>
        <p class="view-subtitle">Review and confirm suggested items</p>
      </div>
      <button type="button" class="ghost-button" @click="store.loadAll()">
        <Icon icon="tabler:refresh" />
        Refresh
      </button>
    </div>

    <!-- Error banner -->
    <div v-if="store.error" class="error-banner">
      <Icon icon="tabler:alert-circle" />
      <span>{{ store.error }}</span>
    </div>

    <!-- Metrics -->
    <div class="review-metrics">
      <div class="metric-card">
        <span class="metric-value">{{ store.totalSuggestedCount }}</span>
        <span class="metric-label">Suggested</span>
      </div>
      <div class="metric-card">
        <span class="metric-value">{{ store.relationsSuggestedCount }}</span>
        <span class="metric-label">Relationships</span>
      </div>
      <div class="metric-card">
        <span class="metric-value">{{ store.decisionsSuggestedCount }}</span>
        <span class="metric-label">Decisions</span>
      </div>
      <div class="metric-card">
        <span class="metric-value">{{ store.obligationsSuggestedCount }}</span>
        <span class="metric-label">Obligations</span>
      </div>
      <div class="metric-card">
        <span class="metric-value">{{ store.contradictionsSuggestedCount }}</span>
        <span class="metric-label">Polygraph</span>
      </div>
    </div>

    <!-- Review Board -->
    <div class="review-board">
      <!-- Relationships -->
      <div class="review-panel">
        <h3 class="panel-title">
          <Icon icon="tabler:users" />
          Relationships
          <span v-if="store.relationsSuggestedCount > 0" class="panel-badge">
            {{ store.relationsSuggestedCount }}
          </span>
        </h3>
        <div v-if="store.relationships.length === 0" class="panel-empty">
          <p>No relationships to review</p>
        </div>
        <div v-else class="panel-items">
          <div
            v-for="item in store.relationships.filter((r) => r.review_state === 'suggested')"
            :key="item.relationship_id"
            class="review-item"
          >
            <div class="item-info">
              <p class="item-desc">{{ relationshipPeer(item) }}</p>
              <p class="item-meta">{{ item.relationship_type }} · Score: {{ item.trust_score?.toFixed(1) }}</p>
            </div>
            <div class="item-actions">
              <button
                type="button"
                class="action-btn confirm"
                :disabled="store.reviewingItemKey === 'relationship:' + item.relationship_id"
                @click="handleReview({ kind: 'relationship', item, reviewState: 'user_confirmed' })"
              >
                <Icon icon="tabler:check" /> Confirm
              </button>
              <button
                type="button"
                class="action-btn reject"
                :disabled="store.reviewingItemKey === 'relationship:' + item.relationship_id"
                @click="handleReview({ kind: 'relationship', item, reviewState: 'user_rejected' })"
              >
                <Icon icon="tabler:x" /> Reject
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Decisions -->
      <div class="review-panel">
        <h3 class="panel-title">
          <Icon icon="tabler:scale" />
          Decisions
          <span v-if="store.decisionsSuggestedCount > 0" class="panel-badge">
            {{ store.decisionsSuggestedCount }}
          </span>
        </h3>
        <div v-if="store.decisions.length === 0" class="panel-empty">
          <p>No decisions to review</p>
        </div>
        <div v-else class="panel-items">
          <div
            v-for="item in store.decisions.filter((d) => d.review_state === 'suggested')"
            :key="item.decision_id"
            class="review-item"
          >
            <div class="item-info">
              <p class="item-desc">{{ item.title }}</p>
              <p class="item-meta">{{ decisionEntityLabel(item) }} · {{ formatItemTime(item.decided_at) }}</p>
            </div>
            <div class="item-actions">
              <button
                type="button"
                class="action-btn confirm"
                :disabled="store.reviewingItemKey === 'decision:' + item.decision_id"
                @click="handleReview({ kind: 'decision', item, reviewState: 'user_confirmed' })"
              >
                <Icon icon="tabler:check" /> Confirm
              </button>
              <button
                type="button"
                class="action-btn reject"
                :disabled="store.reviewingItemKey === 'decision:' + item.decision_id"
                @click="handleReview({ kind: 'decision', item, reviewState: 'user_rejected' })"
              >
                <Icon icon="tabler:x" /> Reject
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Obligations -->
      <div class="review-panel">
        <h3 class="panel-title">
          <Icon icon="tabler:gavel" />
          Obligations
          <span v-if="store.obligationsSuggestedCount > 0" class="panel-badge">
            {{ store.obligationsSuggestedCount }}
          </span>
        </h3>
        <div v-if="store.obligations.length === 0" class="panel-empty">
          <p>No obligations to review</p>
        </div>
        <div v-else class="panel-items">
          <div
            v-for="item in store.obligations.filter((o) => o.review_state === 'suggested')"
            :key="item.obligation_id"
            class="review-item"
          >
            <div class="item-info">
              <p class="item-desc">{{ item.statement }}</p>
              <p class="item-meta">{{ obligationEntityLabel(item) }} · Due: {{ formatItemTime(item.due_at) }}</p>
            </div>
            <div class="item-actions">
              <button
                type="button"
                class="action-btn confirm"
                :disabled="store.reviewingItemKey === 'obligation:' + item.obligation_id"
                @click="handleReview({ kind: 'obligation', item, reviewState: 'user_confirmed' })"
              >
                <Icon icon="tabler:check" /> Confirm
              </button>
              <button
                type="button"
                class="action-btn reject"
                :disabled="store.reviewingItemKey === 'obligation:' + item.obligation_id"
                @click="handleReview({ kind: 'obligation', item, reviewState: 'user_rejected' })"
              >
                <Icon icon="tabler:x" /> Reject
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Polygraph / Contradictions -->
      <div class="review-panel">
        <h3 class="panel-title">
          <Icon icon="tabler:git-compare" />
          Polygraph
          <span v-if="store.contradictionsSuggestedCount > 0" class="panel-badge">
            {{ store.contradictionsSuggestedCount }}
          </span>
        </h3>
        <div v-if="store.contradictions.length === 0" class="panel-empty">
          <p>No contradictions to review</p>
        </div>
        <div v-else class="panel-items">
          <div
            v-for="item in store.contradictions.filter((c) => c.review_state === 'suggested')"
            :key="item.observation_id"
            class="review-item"
          >
            <div class="item-info">
              <p class="item-desc">{{ item.old_claim }} ↔ {{ item.new_claim }}</p>
              <p class="item-meta">{{ item.severity }} · {{ formatItemTime(item.created_at) }}</p>
            </div>
            <div class="item-actions">
              <button
                type="button"
                class="action-btn confirm"
                :disabled="store.reviewingItemKey === 'contradiction:' + item.observation_id"
                @click="handleReview({ kind: 'contradiction', item, reviewState: 'user_confirmed' })"
              >
                <Icon icon="tabler:check" /> Confirm
              </button>
              <button
                type="button"
                class="action-btn reject"
                :disabled="store.reviewingItemKey === 'contradiction:' + item.observation_id"
                @click="handleReview({ kind: 'contradiction', item, reviewState: 'user_rejected' })"
              >
                <Icon icon="tabler:x" /> Reject
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.review-page {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px;
  height: 100%;
  overflow-y: auto;
}

.view-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}

.header-title-group {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.view-title {
  font-size: 20px;
  font-weight: 700;
  margin: 0;
  color: hsl(var(--foreground));
}

.view-subtitle {
  font-size: 13px;
  color: hsl(var(--muted-foreground));
  margin: 0;
}

.ghost-button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 14px;
  font-size: 13px;
  border-radius: 6px;
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  color: hsl(var(--foreground));
  cursor: pointer;
}

.ghost-button:hover {
  background: hsl(var(--accent));
}

.error-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  font-size: 13px;
  color: hsl(var(--destructive));
  background: hsl(var(--destructive) / 0.08);
  border-radius: 8px;
}

.review-metrics {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.metric-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 16px 24px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  min-width: 100px;
}

.metric-value {
  font-size: 28px;
  font-weight: 700;
  color: hsl(var(--foreground));
}

.metric-label {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.review-board {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
  gap: 12px;
}

.review-panel {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  padding: 16px;
}

.panel-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  font-weight: 600;
  margin: 0 0 12px;
  color: hsl(var(--foreground));
}

.panel-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 1px 7px;
  border-radius: 999px;
  background: hsl(var(--primary) / 0.12);
  color: hsl(var(--primary));
  margin-left: auto;
}

.panel-empty {
  padding: 20px 0;
  text-align: center;
  color: hsl(var(--muted-foreground));
  font-size: 13px;
}

.panel-items {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.review-item {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  background: hsl(var(--background));
}

.item-info {
  flex: 1;
  min-width: 0;
}

.item-desc {
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--foreground));
  margin: 0 0 4px;
  line-height: 1.4;
}

.item-meta {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  margin: 0;
}

.item-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 4px 10px;
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

.action-btn.confirm:hover:not(:disabled) {
  background: hsl(var(--success) / 0.1);
  border-color: hsl(var(--success));
}

.action-btn.reject:hover:not(:disabled) {
  background: hsl(var(--destructive) / 0.1);
  border-color: hsl(var(--destructive));
}
</style>
