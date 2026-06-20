<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useReviewStore } from '../stores/review'
import Icon from '../../../shared/ui/Icon.vue'
import type {
	Decision,
	Obligation,
	Relationship,
	ReviewItem,
	ReviewItemPromotionRequest
} from '../types/review'

const store = useReviewStore()
const promoteDrafts = ref<Record<string, ReviewItemPromotionRequest>>({})

onMounted(() => {
	void loadReviewWorkspace()
})

async function loadReviewWorkspace() {
	await store.loadAll()
	syncPromoteDrafts()
}

function relationshipPeer(item: Relationship): string {
	return `${item.source_entity_kind}:${item.source_entity_id} → ${item.target_entity_kind}:${item.target_entity_id}`
}

function decisionEntityLabel(item: Decision): string {
	if (!item.decided_by_entity_kind || !item.decided_by_entity_id) return 'Unknown'
	return `${item.decided_by_entity_kind}:${item.decided_by_entity_id}`
}

function obligationEntityLabel(item: Obligation): string {
	return `${item.obligated_entity_kind}:${item.obligated_entity_id}`
}

function formatItemTime(value: string | null | undefined): string {
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

async function syncPromoteDrafts() {
	store.reviewItems.forEach((item: ReviewItem) => {
		if (promoteDrafts.value[item.review_item_id]) return
		promoteDrafts.value[item.review_item_id] = deriveDefaultPromotion(item)
	})
}

function deriveDefaultPromotion(item: ReviewItem): ReviewItemPromotionRequest {
	const defaults: Record<string, ReviewItemPromotionRequest> = {
		new_person: { target_domain: 'persons', target_entity_kind: 'person', target_entity_id: '' },
		new_organization: {
			target_domain: 'organizations',
			target_entity_kind: 'organization',
			target_entity_id: ''
		},
		potential_task: { target_domain: 'tasks', target_entity_kind: 'task', target_entity_id: '' },
		potential_obligation: {
			target_domain: 'obligations',
			target_entity_kind: 'obligation',
			target_entity_id: ''
		},
		potential_decision: {
			target_domain: 'decisions',
			target_entity_kind: 'decision',
			target_entity_id: ''
		},
		potential_relationship: {
			target_domain: 'relationships',
			target_entity_kind: 'relationship',
			target_entity_id: ''
		},
		potential_project: {
			target_domain: 'projects',
			target_entity_kind: 'project',
			target_entity_id: ''
		},
		knowledge_candidate: {
			target_domain: 'documents',
			target_entity_kind: 'document',
			target_entity_id: `document:review-note:${item.review_item_id}`
		}
	}
	return defaults[item.item_kind] ?? { target_domain: '', target_entity_kind: '', target_entity_id: '' }
}

function canPromote(item: ReviewItem): boolean {
	const draft = promoteDrafts.value[item.review_item_id]
	return !!(
		draft &&
		draft.target_domain.trim() &&
		draft.target_entity_kind.trim() &&
		draft.target_entity_id.trim() &&
		store.reviewingItemKey !== `review_item_promote:${item.review_item_id}`
	)
}

async function handlePromote(item: ReviewItem) {
	const draft = promoteDrafts.value[item.review_item_id]
	if (!draft) return
	await handleReview({
		kind: 'review_item_promote',
		item,
		promotion: { ...draft }
	})
}

function reviewItemButtonPrefix(item: ReviewItem): string {
	return `review_item:${item.review_item_id}`
}

function canArchive(item: ReviewItem): boolean {
	return store.reviewingItemKey !== `review_item_archive:${item.review_item_id}`
}

function reviewItemKindLabel(itemKind: ReviewItem['item_kind']): string {
	return itemKind
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
        <button type="button" class="ghost-button" @click="loadReviewWorkspace()">
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
        <span class="metric-value">{{ store.reviewItemsCount }}</span>
        <span class="metric-label">Review Items</span>
      </div>
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
      <!-- Canonical Review Items -->
      <div class="review-panel">
        <h3 class="panel-title">
          <Icon icon="tabler:inbox" />
          Canonical Inbox
          <span v-if="store.reviewItemsCount > 0" class="panel-badge">
            {{ store.reviewItemsCount }}
          </span>
        </h3>
        <div v-if="store.reviewItems.length === 0" class="panel-empty">
          <p>No canonical review items available</p>
        </div>
        <div v-else class="panel-items">
          <div
            v-for="item in store.reviewItems.filter((r) => r.status === 'new' || r.status === 'in_review')"
            :key="item.review_item_id"
            class="review-item review-item--canonical"
          >
            <div class="item-info">
              <p class="item-desc">{{ item.title }}</p>
              <p class="item-meta">
                {{ reviewItemKindLabel(item.item_kind) }} · {{ item.summary }}
              </p>
              <p class="item-meta">Confidence: {{ item.confidence.toFixed(2) }}</p>
            </div>
            <div class="item-actions item-actions--stacked">
              <div class="action-row">
                <button
                  type="button"
                  class="action-btn confirm"
                  :disabled="store.reviewingItemKey === reviewItemButtonPrefix(item)"
                  @click="handleReview({ kind: 'review_item', item, action: 'approve' })"
                >
				<Icon icon="tabler:check" /> Approve
                </button>
                <button
                  v-if="item.status === 'new'"
                  type="button"
                  class="action-btn"
                  :disabled="store.reviewingItemKey === reviewItemButtonPrefix(item)"
                  @click="handleReview({ kind: 'review_item_take', item })"
                >
                  <Icon icon="tabler:player-play" /> Take
                </button>
                <span v-else class="status-pill">
                  status: {{ item.status }}
                </span>
                <button
                  type="button"
                  class="action-btn reject"
                  :disabled="store.reviewingItemKey === reviewItemButtonPrefix(item)"
                  @click="handleReview({ kind: 'review_item', item, action: 'dismiss' })"
                >
                  <Icon icon="tabler:x" /> Dismiss
                </button>
              </div>
              <div class="action-row">
                <input
                  v-model="promoteDrafts[item.review_item_id].target_domain"
                  type="text"
                  class="review-input"
                  placeholder="target_domain"
                />
                <input
                  v-model="promoteDrafts[item.review_item_id].target_entity_kind"
                  type="text"
                  class="review-input"
                  placeholder="entity_kind"
                />
                <input
                  v-model="promoteDrafts[item.review_item_id].target_entity_id"
                  type="text"
                  class="review-input"
                  placeholder="entity_id"
                />
              </div>
              <div class="action-row">
                <button
                  type="button"
                  class="action-btn promote"
                  :disabled="!canPromote(item)"
                  @click="handlePromote(item)"
                >
                  <Icon icon="tabler:arrow-up-right" /> Promote
                </button>
                <button
                  type="button"
                  class="action-btn archive"
                  :disabled="!canArchive(item)"
                  @click="handleReview({ kind: 'review_item_archive', item })"
                >
                  <Icon icon="tabler:archive" /> Archive
                </button>
                <span v-if="item.target_domain" class="status-pill">
                  promoted: {{ item.target_domain }}/{{ item.target_entity_kind }}/{{ item.target_entity_id }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

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

.item-actions--stacked {
  flex-direction: column;
  align-items: stretch;
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

.action-row {
  display: flex;
  gap: 6px;
}

.review-input {
  flex: 1;
  border-radius: 6px;
  border: 1px solid hsl(var(--border));
  background: hsl(var(--background));
  color: hsl(var(--foreground));
  font-size: 12px;
  padding: 6px;
}

.status-pill {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
}

.action-btn.promote {
  margin-top: 4px;
}
</style>
