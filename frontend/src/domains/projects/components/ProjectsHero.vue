<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { ProjectRecord, ProjectStats, ProjectSummary } from '../types/project'
import { projectStatusLabel, formatProjectDate } from '../stores/projects'

const { t } = useI18n()

const props = defineProps<{
  projectsError: string
  isProjectsLoading: boolean
  selectedProjectRecord: ProjectRecord | null
  selectedProjectStats: ProjectStats
  projectSummaries: ProjectSummary[]
  selectProject: (item: ProjectSummary) => void
  loadProjects: () => void
}>()
</script>

<template>
  <!-- Error state -->
  <div v-if="props.projectsError && !props.selectedProjectRecord" class="widget-frame">
    <section class="panel info-card project-empty-state">
      <span class="icon-placeholder">⚠</span>
      <h2>{{ t('Projects unavailable') }}</h2>
      <p>{{ props.projectsError }}</p>
      <button type="button" class="primary-button" @click="props.loadProjects">{{ t('Retry') }}</button>
    </section>
  </div>

  <!-- Empty state -->
  <div v-else-if="!props.selectedProjectRecord" class="widget-frame">
    <section class="panel info-card project-empty-state">
      <span class="icon-placeholder">◻</span>
      <h2>{{ t('No projects returned') }}</h2>
      <p>{{ props.isProjectsLoading ? t('Loading local projects...') : t('Local project records are empty.') }}</p>
    </section>
  </div>

  <!-- Projects loaded -->
  <template v-else>
    <div class="widget-frame">
      <header class="project-hero panel">
        <div class="project-logo"><Icon icon="tabler:cube" :size="48" /></div>
        <div>
          <h1>{{ props.selectedProjectRecord.name }} <em>{{ projectStatusLabel(props.selectedProjectRecord.status) }}</em></h1>
          <p>{{ props.selectedProjectRecord.kind }}</p>
          <small>{{ props.selectedProjectRecord.description }}</small>
        </div>
        <button type="button" class="primary-button" disabled>
          <Icon icon="tabler:calendar-stats" :size="16" /> {{ t('Prepare brief') }}
        </button>
      </header>
    </div>

    <div class="widget-frame">
      <div class="project-meta-strip panel">
        <article><span>{{ t('Owner') }}</span><strong>{{ props.selectedProjectRecord.owner_display_name }}</strong></article>
        <article><span>{{ t('People') }}</span><strong>{{ props.selectedProjectStats.people_count }}</strong></article>
        <article><span>{{ t('Start Date') }}</span><strong>{{ formatProjectDate(props.selectedProjectRecord.start_date) }}</strong></article>
        <article><span>{{ t('Target Date') }}</span><strong>{{ formatProjectDate(props.selectedProjectRecord.target_date) }}</strong></article>
        <article>
          <span>{{ t('Progress') }}</span>
          <progress class="progress" :max="100" :value="props.selectedProjectRecord.progress_percent" :aria-label="`${props.selectedProjectRecord.name} progress`" />
          <strong>{{ props.selectedProjectRecord.progress_percent }}%</strong>
        </article>
      </div>
    </div>

    <div v-if="props.projectSummaries.length > 1" class="widget-frame">
      <div class="project-switcher panel">
        <button
          v-for="item in props.projectSummaries"
          :key="item.project.project_id"
          type="button"
          :class="{ active: item.project.project_id === props.selectedProjectRecord.project_id }"
          @click="props.selectProject(item)"
        >
          <Icon icon="tabler:cube" :size="16" />
          <span>{{ item.project.name }}</span>
          <em>{{ item.project.progress_percent }}%</em>
        </button>
      </div>
    </div>

    <div class="widget-frame">
      <div class="section-tabs">
        <button type="button" class="active">{{ t('Overview') }}</button>
        <button type="button" disabled>{{ t('Communications') }} <em>{{ props.selectedProjectStats.message_count }}</em></button>
        <button type="button" disabled>{{ t('Tasks') }}</button>
        <button type="button" disabled>{{ t('Documents') }} <em>{{ props.selectedProjectStats.document_count }}</em></button>
        <button type="button" disabled>{{ t('Calendar') }}</button>
        <button type="button" disabled>{{ t('Team') }} <em>{{ props.selectedProjectStats.people_count }}</em></button>
        <button type="button" disabled>{{ t('Notes') }}</button>
        <button type="button" disabled>{{ t('Files') }}</button>
        <button type="button" disabled>{{ t('Settings') }}</button>
      </div>
    </div>

    <p v-if="props.projectsError" class="inline-error">{{ props.projectsError }}</p>
  </template>
</template>
