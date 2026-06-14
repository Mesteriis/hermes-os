<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import type { ProjectDetail, ProjectRecord, ProjectStats, ProjectSummary } from '../types/project'
import { projectStatusLabel } from '../stores/projects'

const { t } = useI18n()

const props = defineProps<{
  selectedProjectDetail: ProjectDetail | null
  selectedProjectRecord: ProjectRecord
  selectedProjectStats: ProjectStats
  relatedProjectSummaries: ProjectSummary[]
  formatNumber: (num: number) => string
}>()
</script>

<template>
  <aside class="stacked-rail project-side">
    <!-- Project Health -->
    <div class="widget-frame">
      <section class="panel info-card">
        <h2>{{ t('Project Health') }}</h2>
        <div class="health-row"><span>{{ t('Status') }}</span><strong>{{ projectStatusLabel(props.selectedProjectRecord.status) }}</strong></div>
        <div class="health-row"><span>{{ t('Progress') }}</span><strong>{{ props.selectedProjectRecord.progress_percent }}%</strong></div>
        <div class="health-row"><span>{{ t('Graph Links') }}</span><strong>{{ props.formatNumber(props.selectedProjectStats.graph_connection_count) }}</strong></div>
      </section>
    </div>

    <!-- Key People -->
    <div class="widget-frame">
      <section class="panel info-card">
        <h2>{{ t('Key People') }}</h2>
        <template v-if="props.selectedProjectDetail?.key_people.length">
          <div v-for="person in props.selectedProjectDetail.key_people" :key="person.email_address" class="person-compact">
            <span class="round-icon ghost"><Icon icon="tabler:user" :size="16" /></span>
            <span><strong>{{ person.display_name }}</strong><small>{{ person.email_address }}</small></span>
            <em>{{ props.formatNumber(person.interaction_count) }}</em>
          </div>
        </template>
        <p v-else class="muted-copy">{{ t('No linked people.') }}</p>
      </section>
    </div>

    <!-- Related Projects -->
    <div class="widget-frame">
      <section class="panel info-card">
        <h2>{{ t('Related Projects') }}</h2>
        <template v-if="props.relatedProjectSummaries.length">
          <div v-for="item in props.relatedProjectSummaries.slice(0, 4)" :key="item.project.project_id" class="related-row">
            <span class="round-icon cyan"><Icon icon="tabler:cube" :size="16" /></span>
            <strong>{{ item.project.name }}</strong>
            <em>{{ item.project.progress_percent }}%</em>
          </div>
        </template>
        <p v-else class="muted-copy">{{ t('No related project records.') }}</p>
      </section>
    </div>
  </aside>
</template>
