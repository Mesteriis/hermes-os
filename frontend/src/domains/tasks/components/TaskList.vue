<script setup lang="ts">
import { ref, computed } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useI18n } from '../../../platform/i18n'
import type { Task, TaskCandidate, TaskCandidateReviewState } from '../types/task'
import { taskSourceLabel, taskConfidence, taskCreatedTime } from '../stores/tasks'

const { t } = useI18n()

const props = defineProps<{
  activeTasks: Task[]
  suggestedTaskCandidates: TaskCandidate[]
  isTasksLoading: boolean
  setTaskCandidateReview: (candidate: TaskCandidate, state: TaskCandidateReviewState) => Promise<void>
}>()

const parentRef = ref<HTMLElement | null>(null)

// Combine active tasks and suggested candidates into one virtual list
const allRows = computed<(Task | TaskCandidate)[]>(() => {
  const separator: TaskCandidate = {
    task_candidate_id: '__separator__',
    source_kind: 'message',
    source_id: '',
    project_id: null,
    title: t('Review Queue'),
    due_text: null,
    assignee_label: null,
    confidence: 0,
    review_state: 'suggested',
    evidence_excerpt: '',
    generated_at: '',
    reviewed_at: null,
    updated_at: ''
  }
  return [...props.activeTasks, separator, ...props.suggestedTaskCandidates]
})

const virtualizer = useVirtualizer(computed(() => ({
  count: allRows.value.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 52,
  overscan: 5
})))

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())

function isCandidate(item: Task | TaskCandidate): item is TaskCandidate {
  return 'task_candidate_id' in item && 'confidence' in item && 'review_state' in item
}

function isSeparator(item: Task | TaskCandidate): boolean {
  return 'task_candidate_id' in item && item.task_candidate_id === '__separator__'
}

function isTask(item: Task | TaskCandidate): item is Task {
  return 'task_id' in item && !('confidence' in item)
}
</script>

<template>
  <div class="widget-frame">
    <section class="panel task-table">
      <h3 class="task-group">{{ t('Active Tasks') }} <em>{{ props.activeTasks.length }}</em></h3>

      <div class="table-head task-table-head">
        <span>{{ t('Task') }}</span>
        <span>{{ t('Source') }}</span>
        <span>{{ t('Project') }}</span>
        <span>{{ t('Created') }}</span>
        <span>{{ t('Status') }}</span>
      </div>

      <!-- Loading state -->
      <div v-if="props.isTasksLoading" class="inline-copy">
        {{ t('Loading task state…') }}
      </div>

      <!-- Empty state -->
      <div v-else-if="props.activeTasks.length === 0 && props.suggestedTaskCandidates.length === 0" class="inline-copy">
        {{ t('No active tasks yet.') }}
      </div>

      <!-- Virtual list -->
      <div v-else ref="parentRef" class="virtual-list-container" style="overflow-y: auto; max-height: 600px;">
        <div class="virtual-list-inner" :style="{ height: `${totalSize}px`, width: '100%', position: 'relative' }">
          <div
            v-for="virtualRow in virtualItems"
            :key="(allRows[virtualRow.index] as any).task_id || (allRows[virtualRow.index] as any).task_candidate_id || virtualRow.index"
            :style="{
              position: 'absolute',
              top: 0,
              left: 0,
              width: '100%',
              height: `${virtualRow.size}px`,
              transform: `translateY(${virtualRow.start}px)`
            }"
          >
            <!-- Separator row -->
            <div v-if="isSeparator(allRows[virtualRow.index])" class="task-group" style="padding: 8px 12px;">
              <h3>{{ t('Review Queue') }} <em>{{ props.suggestedTaskCandidates.length }}</em></h3>
            </div>

            <!-- Active task row -->
            <label v-else-if="isTask(allRows[virtualRow.index])" class="task-row">
              <input type="checkbox" disabled checked />
              <strong>{{ (allRows[virtualRow.index] as Task).title }}</strong>
              <span>{{ taskSourceLabel(allRows[virtualRow.index] as Task) }}</span>
              <span>{{ (allRows[virtualRow.index] as Task).project_id ?? t('Unassigned') }}</span>
              <time>{{ taskCreatedTime((allRows[virtualRow.index] as Task).created_at) }}</time>
              <em>{{ (allRows[virtualRow.index] as Task).hermes_status }}</em>
            </label>

            <!-- Candidate row -->
            <div v-else class="task-row task-row-actions">
              <strong>{{ (allRows[virtualRow.index] as TaskCandidate).title }}</strong>
              <span>{{ taskSourceLabel(allRows[virtualRow.index] as TaskCandidate) }}</span>
              <span>{{ (allRows[virtualRow.index] as TaskCandidate).project_id ?? t('Unassigned') }}</span>
              <em>{{ taskConfidence(allRows[virtualRow.index] as TaskCandidate) }}</em>
              <div class="task-actions">
                <button type="button" @click="props.setTaskCandidateReview(allRows[virtualRow.index] as TaskCandidate, 'user_confirmed')">
                  <Icon icon="tabler:check" :size="15" /> {{ t('Confirm') }}
                </button>
                <button type="button" @click="props.setTaskCandidateReview(allRows[virtualRow.index] as TaskCandidate, 'user_rejected')">
                  <Icon icon="tabler:x" :size="15" /> {{ t('Reject') }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>
