<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import type { DocumentProcessingJob } from '../types/documents'

const { t } = useI18n()

defineProps<{
  jobs: DocumentProcessingJob[]
  isLoading: boolean
  detailError: string
  retryingJobId: string | null
}>()

const emit = defineEmits<{
  retry: [job: DocumentProcessingJob]
}>()
</script>

<template>
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Processing Jobs') }}</h2>
      <div v-if="isLoading" class="graph-strip-message">
        <span>{{ t('Loading jobs.') }}</span>
      </div>
      <template v-else>
        <div v-for="job in jobs.slice(0, 5)" :key="job.job_id" class="job-row">
          <strong>{{ job.document_id }}</strong>
          <span :class="['status-chip', job.status]">{{ job.status }}</span>
          <small>{{ job.step }} &middot; {{ job.queued_at }}</small>
          <button
            v-if="job.status === 'failed'"
            type="button"
            :disabled="retryingJobId === job.document_id"
            @click="emit('retry', job)"
          >
            {{ retryingJobId === job.document_id ? t('Retrying...') : t('Retry') }}
          </button>
        </div>
        <p v-if="detailError" class="inline-error">{{ detailError }}</p>
      </template>
    </section>
  </div>
</template>
