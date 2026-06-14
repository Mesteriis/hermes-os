<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type {
  Decision,
  DecisionEntityKind,
  Obligation
} from '../types/task'
import { formatDecisionTime, formatDecisionEntity, formatObligationDueTime, formatObligationEntity } from '../stores/tasks'

const { t } = useI18n()

const entityKindOptions: DecisionEntityKind[] = [
  'project', 'task', 'persona', 'communication',
  'document', 'event', 'organization', 'knowledge'
]

const props = defineProps<{
  decisions: Decision[]
  obligations: Obligation[]
  entityKind: DecisionEntityKind
  entityId: string
  isLoading: boolean
  error: string
  reviewingItemId: string | null
  onEntityKindChange: (entityKind: DecisionEntityKind) => void
  onEntityIdChange: (entityId: string) => void
  onReload: () => Promise<void>
  onReviewDecision: (decision: Decision, reviewState: 'user_confirmed' | 'user_rejected') => Promise<void>
  onReviewObligation: (obligation: Obligation, reviewState: 'user_confirmed' | 'user_rejected') => Promise<void>
}>()

const hasScope = computed(() => props.entityId.trim().length > 0)
const reviewItemCount = computed(() => props.decisions.length + props.obligations.length)

function decisionReviewId(decision: Decision): string {
  return `decision:${decision.decision_id}`
}

function obligationReviewId(obligation: Obligation): string {
  return `obligation:${obligation.obligation_id}`
}
</script>

<template>
  <section class="widget-frame task-context-review-panel" :aria-busy="props.isLoading">
    <header>
      <div>
        <span class="panel-kicker">{{ t('Context') }}</span>
        <h2>{{ t('Decision & Obligation Review') }}</h2>
      </div>
      <button type="button" :title="t('Reload review items')" @click="props.onReload" :disabled="props.isLoading">
        <Icon icon="tabler:refresh" :size="15" />
      </button>
    </header>

    <div class="task-context-review-scope">
      <select
        :aria-label="t('Entity kind')"
        :value="props.entityKind"
        @change="props.onEntityKindChange(($event.target as HTMLSelectElement).value as DecisionEntityKind)"
      >
        <option v-for="option in entityKindOptions" :key="option" :value="option">{{ t(option) }}</option>
      </select>
      <input
        :aria-label="t('Entity id')"
        :value="props.entityId"
        :placeholder="t('Entity id')"
        @input="props.onEntityIdChange(($event.target as HTMLInputElement).value)"
      />
    </div>

    <!-- Error state -->
    <div v-if="props.error" class="task-context-review-state error">
      <span>{{ props.error }}</span>
      <button type="button" @click="props.onReload" :disabled="props.isLoading">{{ t('Retry') }}</button>
    </div>

    <!-- Loading state -->
    <div v-else-if="props.isLoading" class="task-context-review-state">
      <span>{{ hasScope ? t('Loading review items') : t('Loading global review items') }}</span>
    </div>

    <!-- Empty state -->
    <div v-else-if="reviewItemCount === 0" class="task-context-review-state">
      <span>{{ t('No open decisions or obligations') }}</span>
    </div>

    <!-- Review list -->
    <div v-else class="task-context-review-list">
      <article v-for="decision in props.decisions" :key="decision.decision_id" class="task-context-review-item">
        <div>
          <span class="panel-kicker">{{ t('Decision') }}</span>
          <strong>{{ decision.title }}</strong>
          <p>{{ decision.rationale }}</p>
          <small>{{ formatDecisionEntity(decision.decided_by_entity_kind, decision.decided_by_entity_id) }} · {{ formatDecisionTime(decision.decided_at) }}</small>
        </div>
        <div class="task-context-review-actions">
          <button
            type="button"
            :disabled="props.reviewingItemId === decisionReviewId(decision)"
            @click="props.onReviewDecision(decision, 'user_confirmed')"
          >
            <Icon icon="tabler:check" :size="14" /> {{ t('Confirm') }}
          </button>
          <button
            type="button"
            :disabled="props.reviewingItemId === decisionReviewId(decision)"
            @click="props.onReviewDecision(decision, 'user_rejected')"
          >
            <Icon icon="tabler:x" :size="14" /> {{ t('Reject') }}
          </button>
        </div>
      </article>

      <article v-for="obligation in props.obligations" :key="obligation.obligation_id" class="task-context-review-item">
        <div>
          <span class="panel-kicker">{{ t('Obligation') }}</span>
          <strong>{{ obligation.statement }}</strong>
          <p>{{ formatObligationEntity(obligation.obligated_entity_kind, obligation.obligated_entity_id) }}</p>
          <small>{{ t(obligation.risk_state) }} · {{ formatObligationDueTime(obligation.due_at) }}</small>
        </div>
        <div class="task-context-review-actions">
          <button
            type="button"
            :disabled="props.reviewingItemId === obligationReviewId(obligation)"
            @click="props.onReviewObligation(obligation, 'user_confirmed')"
          >
            <Icon icon="tabler:check" :size="14" /> {{ t('Confirm') }}
          </button>
          <button
            type="button"
            :disabled="props.reviewingItemId === obligationReviewId(obligation)"
            @click="props.onReviewObligation(obligation, 'user_rejected')"
          >
            <Icon icon="tabler:x" :size="14" /> {{ t('Reject') }}
          </button>
        </div>
      </article>
    </div>
  </section>
</template>
