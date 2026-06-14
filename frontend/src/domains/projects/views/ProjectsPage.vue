<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useProjectsQuery, useProjectQuery } from '../queries/useProjectsQuery'
import { useProjectsStore } from '../stores/projects'
import ProjectsHero from '../components/ProjectsHero.vue'
import ProjectsDashboard from '../components/ProjectsDashboard.vue'
import ProjectsRail from '../components/ProjectsRail.vue'
import type { ProjectSummary, ProjectDetail } from '../types/project'

const { t } = useI18n()
const store = useProjectsStore()

const { data: projectsData, isLoading: isProjectsLoading, error: projectsErrorObj, refetch: refetchProjects } = useProjectsQuery()
const { data: projectDetailData, isLoading: isDetailLoading } = useProjectQuery(store.selectedProjectId || null)

const projectsError = computed<string>(() => {
  if (projectsErrorObj.value) return projectsErrorObj.value?.message ?? t('Unknown projects error')
  return ''
})

const projectSummaries = computed<ProjectSummary[]>(() => {
  return projectsData.value ?? []
})

const selectedProjectDetail = computed<ProjectDetail | null>(() => {
  return projectDetailData.value ?? null
})

const selectedProjectRecord = computed(() => {
  return selectedProjectDetail.value?.project ?? projectSummaries.value[0]?.project ?? null
})

const selectedProjectStats = computed(() => {
  return selectedProjectDetail.value?.stats ?? projectSummaries.value[0]?.stats ?? { message_count: 0, document_count: 0, people_count: 0, graph_connection_count: 0, latest_activity_at: null }
})

const relatedProjectSummaries = computed<ProjectSummary[]>(() => {
  const currentId = selectedProjectRecord.value?.project_id
  return projectSummaries.value.filter((item) => item.project.project_id !== currentId)
})

function selectProject(item: ProjectSummary) {
  if (item.project.project_id === store.selectedProjectId && selectedProjectDetail.value) return
  store.selectProject(item.project.project_id)
}

function loadProjects() {
  refetchProjects()
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat('en-US').format(value)
}
</script>

<template>
  <section class="projects-page">
    <ProjectsHero
      :projectsError="projectsError"
      :isProjectsLoading="isProjectsLoading"
      :selectedProjectRecord="selectedProjectRecord"
      :selectedProjectStats="selectedProjectStats"
      :projectSummaries="projectSummaries"
      :selectProject="selectProject"
      :loadProjects="loadProjects"
    />

    <div v-if="selectedProjectRecord" class="project-dashboard-grid">
      <ProjectsDashboard
        :selectedProjectDetail="selectedProjectDetail"
        :selectedProjectRecord="selectedProjectRecord"
        :selectedProjectStats="selectedProjectStats"
        :formatNumber="formatNumber"
      />
      <ProjectsRail
        :selectedProjectDetail="selectedProjectDetail"
        :selectedProjectRecord="selectedProjectRecord"
        :selectedProjectStats="selectedProjectStats"
        :relatedProjectSummaries="relatedProjectSummaries"
        :formatNumber="formatNumber"
      />
    </div>
  </section>
</template>
