<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import type { ProjectDetail, ProjectRecord, ProjectStats, ProjectTimelineItem, ProjectMessageSummary, ProjectDocumentSummary } from '../types/project'
import { projectTimelineIcon, projectDocumentIcon, formatProjectDateTime } from '../stores/projects'

const { t } = useI18n()

const props = defineProps<{
  selectedProjectDetail: ProjectDetail | null
  selectedProjectRecord: ProjectRecord
  selectedProjectStats: ProjectStats
  formatNumber: (num: number) => string
}>()

function projectMessageSender(message: ProjectMessageSummary): string {
  return message.sender || t('Unknown')
}
</script>

<template>
  <!-- Project Summary -->
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Project Summary') }}</h2>
      <div class="summary-numbers">
        <article><strong>{{ props.formatNumber(props.selectedProjectStats.document_count) }}</strong><span>{{ t('Documents') }}</span></article>
        <article><strong>{{ props.formatNumber(props.selectedProjectStats.message_count) }}</strong><span>{{ t('Messages') }}</span></article>
        <article><strong>{{ props.formatNumber(props.selectedProjectStats.people_count) }}</strong><span>{{ t('People') }}</span></article>
        <article><strong>{{ props.formatNumber(props.selectedProjectStats.graph_connection_count) }}</strong><span>{{ t('Graph links') }}</span></article>
      </div>
    </section>
  </div>

  <!-- Knowledge Graph -->
  <div class="widget-frame">
    <section class="panel graph-card-large">
      <h2>{{ t('Knowledge Graph') }}</h2>
      <div class="radial-graph">
        <div class="graph-center"><Icon icon="tabler:cube" :size="30" /><span>{{ props.selectedProjectRecord.name }}</span></div>
        <span class="graph-chip graph-chip-messages">{{ t('Messages') }} {{ props.formatNumber(props.selectedProjectStats.message_count) }}</span>
        <span class="graph-chip graph-chip-documents">{{ t('Documents') }} {{ props.formatNumber(props.selectedProjectStats.document_count) }}</span>
        <span class="graph-chip graph-chip-people">{{ t('People') }} {{ props.formatNumber(props.selectedProjectStats.people_count) }}</span>
        <span class="graph-chip graph-chip-links">{{ t('Links') }} {{ props.formatNumber(props.selectedProjectStats.graph_connection_count) }}</span>
      </div>
    </section>
  </div>

  <!-- Project Timeline -->
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Project Timeline') }}</h2>
      <template v-if="props.selectedProjectDetail?.timeline.length">
        <div v-for="item in props.selectedProjectDetail.timeline" :key="item.item_id" class="timeline-mini">
          <Icon :icon="projectTimelineIcon(item.item_kind)" :size="16" />
          <time>{{ formatProjectDateTime(item.occurred_at) }}</time>
          <strong>{{ item.title }}</strong>
        </div>
      </template>
      <p v-else class="muted-copy">{{ t('No timeline items from local sources.') }}</p>
    </section>
  </div>

  <!-- Recent Communications -->
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Recent Communications') }}</h2>
      <template v-if="props.selectedProjectDetail?.recent_messages.length">
        <div v-for="message in props.selectedProjectDetail.recent_messages" :key="message.message_id" class="related-row">
          <span class="round-icon cyan"><Icon icon="tabler:mail" :size="16" /></span>
          <strong>{{ projectMessageSender(message) }}</strong>
          <em>{{ formatProjectDateTime(message.occurred_at) }}</em>
        </div>
      </template>
      <p v-else class="muted-copy">{{ t('No linked communications.') }}</p>
    </section>
  </div>

  <!-- Top Documents -->
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Top Documents') }}</h2>
      <template v-if="props.selectedProjectDetail?.documents.length">
        <div v-for="document in props.selectedProjectDetail.documents" :key="document.document_id" class="doc-mini">
          <Icon :icon="projectDocumentIcon(document.document_kind)" :size="20" />
          <span><strong>{{ document.title }}</strong><small>{{ document.document_kind }} · {{ formatProjectDateTime(document.imported_at) }}</small></span>
        </div>
      </template>
      <p v-else class="muted-copy">{{ t('No linked documents.') }}</p>
    </section>
  </div>

  <!-- Source Evidence -->
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Source Evidence') }}</h2>
      <div class="summary-numbers compact">
        <article><strong>{{ props.formatNumber(props.selectedProjectStats.message_count + props.selectedProjectStats.document_count) }}</strong><span>{{ t('Matched records') }}</span></article>
        <article><strong>{{ formatProjectDateTime(props.selectedProjectStats.latest_activity_at) }}</strong><span>{{ t('Last activity') }}</span></article>
      </div>
    </section>
  </div>

  <!-- Open Promises -->
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Open Promises') }}</h2>
      <p class="muted-copy">{{ t('No task candidates connected to this project.') }}</p>
      <button type="button" class="link-row" disabled>{{ t('View all promises') }} <Icon icon="tabler:arrow-right" :size="15" /></button>
    </section>
  </div>
</template>
