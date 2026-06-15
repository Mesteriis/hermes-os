<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useTaskCandidatesQuery, useTasksQuery } from '../queries/useTasksQuery'
import { useTasksStore } from '../stores/tasks'
import { fetchDecisions, fetchDecisionReviewItems, reviewDecision, fetchObligations, fetchObligationReviewItems, reviewObligation } from '../api/tasks'
import { reviewTaskCandidate } from '../api/tasks'
import TaskList from '../components/TaskList.vue'
import TasksDecisionObligationReview from '../components/TasksDecisionObligationReview.vue'
import type { TaskCandidate, Task, Decision, Obligation, DecisionEntityKind, TaskCandidateReviewState } from '../types/task'

const { t } = useI18n()
const store = useTasksStore()

const { data: candidatesData, isLoading: isCandidatesLoading, refetch: refetchCandidates } = useTaskCandidatesQuery()
const { data: tasksData, isLoading: isTasksLoading, refetch: refetchTasks } = useTasksQuery()

// Derived state
const taskCandidates = computed<TaskCandidate[]>(() => candidatesData.value ?? [])
const activeTasks = computed<Task[]>(() => tasksData.value ?? [])
const isTasksLoadingCombined = computed<boolean>(() => isCandidatesLoading.value || isTasksLoading.value)

const suggestedTaskCandidates = computed<TaskCandidate[]>(() =>
  taskCandidates.value.filter((item) => item.review_state === 'suggested')
)

async function loadContextReview() {
  const entityId = store.reviewEntityId.trim()
  store.setContextReviewLoading(true)

  try {
    let decisionResult: { items: Decision[] }
    let obligationResult: { items: Obligation[] }

    if (entityId) {
      ;[decisionResult, obligationResult] = await Promise.all([
        fetchDecisions({ entityKind: store.reviewEntityKind, entityId, limit: 50 }),
        fetchObligations({ entityKind: store.reviewEntityKind as any, entityId, limit: 50 })
      ])
    } else {
      ;[decisionResult, obligationResult] = await Promise.all([
        fetchDecisionReviewItems({ reviewState: 'suggested', limit: 50 }),
        fetchObligationReviewItems({ reviewState: 'suggested', limit: 50 })
      ])
    }
    store.setDecisions(decisionResult.items)
    store.setObligations(obligationResult.items)
    store.setContextReviewError('')
  } catch (e: unknown) {
    store.setDecisions([])
    store.setObligations([])
    store.setContextReviewError(e instanceof Error ? e.message : 'Unknown context review error')
  }
  store.setContextReviewLoading(false)
}

async function reviewDecisionItem(decision: Decision, reviewState: 'user_confirmed' | 'user_rejected') {
  store.setReviewingItemId(`decision:${decision.decision_id}`)
  try {
    await reviewDecision(decision.decision_id, { review_state: reviewState })
    await loadContextReview()
  } catch (e: unknown) {
    store.setContextReviewError(e instanceof Error ? e.message : 'Unknown decision review error')
  }
  store.setReviewingItemId(null)
}

async function reviewObligationItem(obligation: Obligation, reviewState: 'user_confirmed' | 'user_rejected') {
  store.setReviewingItemId(`obligation:${obligation.obligation_id}`)
  try {
    await reviewObligation(obligation.obligation_id, { review_state: reviewState })
    await loadContextReview()
  } catch (e: unknown) {
    store.setContextReviewError(e instanceof Error ? e.message : 'Unknown obligation review error')
  }
  store.setReviewingItemId(null)
}

async function loadTasks() {
  await Promise.all([refetchCandidates(), refetchTasks()])
}

async function setTaskCandidateReview(candidate: TaskCandidate, state: TaskCandidateReviewState) {
  try {
    await reviewTaskCandidate(candidate.task_candidate_id, state)
    store.clearError()
    await loadTasks()
  } catch (e: unknown) {
    store.setError(e instanceof Error ? e.message : 'Unknown task candidate review error')
  }
}

onMounted(() => {
  loadContextReview()
})
</script>

<template>
  <section class="tasks-page">
    <div class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small"><Icon icon="tabler:hexagon" :size="28" /></span>
        <div>
          <h1>{{ t('Tasks') }}</h1>
          <p>{{ t('All your tasks from connected trackers') }}</p>
        </div>
      </div>

      <!-- Metrics -->
      <div class="widget-frame inline-metrics">
        <div class="metric-grid inline-metrics">
          <article class="metric-card">
            <span>{{ t('Active Tasks') }}</span>
            <strong>{{ activeTasks.length }}</strong>
            <small>{{ t('Active records') }}</small>
          </article>
          <article class="metric-card">
            <span>{{ t('Suggested Candidates') }}</span>
            <strong>{{ suggestedTaskCandidates.length }}</strong>
            <small>{{ t('Ready for review') }}</small>
          </article>
          <article class="metric-card">
            <span>{{ t('Review State') }}</span>
            <strong>{{ store.tasksError ? t('Error') : t('Ready') }}</strong>
            <small>{{ store.tasksError ? t('Show message below') : t('Live API') }}</small>
          </article>
        </div>
      </div>

      <button type="button" class="primary-button" disabled>
        <Icon icon="tabler:sparkles" :size="16" />{{ t('AI refresh') }}
      </button>
    </div>

    <p v-if="store.tasksError" class="inline-error">{{ store.tasksError }}</p>

    <div class="tasks-layout">
      <TaskList
        :activeTasks="activeTasks"
        :suggestedTaskCandidates="suggestedTaskCandidates"
        :isTasksLoading="isTasksLoadingCombined"
        :setTaskCandidateReview="setTaskCandidateReview"
      />

      <aside class="stacked-rail">
        <!-- Review Stats -->
        <div class="widget-frame">
          <section class="panel chart-panel">
            <h2>{{ t('Review Stats') }}</h2>
            <div class="donut">
              <strong>{{ taskCandidates.length }}</strong>
              <span>{{ t('Suggestions') }}</span>
            </div>
            <ul>
              <li>{{ suggestedTaskCandidates.length }} {{ t('Suggested') }}</li>
              <li>{{ activeTasks.length }} {{ t('Active') }}</li>
              <li>{{ taskCandidates.length - suggestedTaskCandidates.length - activeTasks.length }} {{ t('Done') }}</li>
            </ul>
          </section>
        </div>

        <!-- Decision & Obligation Review -->
        <TasksDecisionObligationReview
          :decisions="store.decisions"
          :obligations="store.obligations"
          :entityKind="store.reviewEntityKind"
          :entityId="store.reviewEntityId"
          :isLoading="store.isContextReviewLoading"
          :error="store.contextReviewError"
          :reviewingItemId="store.reviewingContextItemId"
          :onEntityKindChange="store.setReviewEntityKind"
          :onEntityIdChange="store.setReviewEntityId"
          :onReload="loadContextReview"
          :onReviewDecision="reviewDecisionItem"
          :onReviewObligation="reviewObligationItem"
        />

        <!-- Recent Candidate Signals -->
        <div class="widget-frame">
          <section class="panel info-card">
            <h2>{{ t('Recent Candidate Signals') }}</h2>
            <template v-if="suggestedTaskCandidates.length === 0">
              <p class="muted-copy">{{ t('No pending candidate signals.') }}</p>
            </template>
            <template v-else>
              <div v-for="candidate in suggestedTaskCandidates.slice(0, 5)" :key="candidate.task_candidate_id" class="deadline">
                <span>{{ candidate.title }}</span>
                <time>{{ candidate.source_kind }}</time>
              </div>
            </template>
          </section>
        </div>

        <!-- Active Task Sources -->
        <div class="widget-frame">
          <section class="panel info-card">
            <h2>{{ t('Active Task Sources') }}</h2>
            <div v-for="source in ['message', 'document']" :key="source" class="bar-row">
              <span>{{ source }}</span>
              <div><i></i></div>
            </div>
          </section>
        </div>
      </aside>
    </div>
  </section>
</template>
