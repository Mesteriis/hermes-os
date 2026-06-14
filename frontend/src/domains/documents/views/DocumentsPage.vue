<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useDocumentsStore } from '../stores/documents'
import { useDocumentProcessingJobsQuery } from '../queries/useDocumentsQuery'
import { retryDocumentProcessingJob } from '../api/documents'
import type { DocumentProcessingJob, DocDisplayItem } from '../types/documents'
import DocumentsSourceCards from '../components/DocumentsSourceCards.vue'
import DocumentsNavigation from '../components/DocumentsNavigation.vue'
import DocumentsList from '../components/DocumentsList.vue'
import DocumentsProcessingJobs from '../components/DocumentsProcessingJobs.vue'
import DocumentsInsights from '../components/DocumentsInsights.vue'

const { t } = useI18n()
const store = useDocumentsStore()

const { data: jobsData, isLoading, refetch: refetchJobs } = useDocumentProcessingJobsQuery(50)

const documentProcessingJobs = computed(() => jobsData.value?.items ?? [])

const documents = computed<DocDisplayItem[]>(() =>
  documentProcessingJobs.value.map((job) => ({
    name: `${job.document_id} (${job.step})`,
    source: 'Hermes Hub',
    project: job.status,
    type: job.step,
    date: job.queued_at,
    size: job.last_error_summary || 'No errors',
    icon: 'tabler:file-text',
    tone: job.status === 'succeeded' ? 'green' : job.status === 'failed' ? 'red' : 'amber'
  }))
)

async function handleRetry(job: DocumentProcessingJob) {
  if (store.retryingJobId === job.job_id) return
  store.setRetryingJobId(job.job_id)
  store.setDocumentsError('')
  try {
    await retryDocumentProcessingJob(job.job_id, {
      command_id: `document-processing-retry-${Date.now()}-${job.job_id}`
    })
  } catch (e) {
    store.setDocumentsError(e instanceof Error ? e.message : 'Retry failed')
  }
  await refetchJobs()
  store.setRetryingJobId(null)
}

onMounted(() => {
  refetchJobs()
})
</script>

<template>
  <section class="documents-page">
    <div class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small"><Icon icon="tabler:file-text" :size="28" /></span>
        <div>
          <h1>{{ t('Documents') }}</h1>
          <p>{{ t('All your documents from connected sources') }}</p>
        </div>
      </div>
    </div>
    <div class="documents-layout">
      <DocumentsSourceCards />
      <DocumentsNavigation />
      <DocumentsList
        :documents="documents"
        :search-query="store.searchQuery"
        :active-filter="store.activeFilter"
        @update:search-query="store.setSearchQuery"
        @update:active-filter="store.setActiveFilter"
      />
      <aside class="stacked-rail">
        <DocumentsProcessingJobs
          :jobs="documentProcessingJobs"
          :is-loading="isLoading"
          :detail-error="store.documentsError"
          :retrying-job-id="store.retryingJobId"
          @retry="handleRetry"
        />
        <DocumentsInsights />
      </aside>
    </div>
  </section>
</template>
