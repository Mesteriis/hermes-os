<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import {
  remoteImageUrlsFromHtml,
  renderMessageBody,
  rewriteRemoteImageSources
} from '../../../shared/sanitize/emailHtml'
import { loadFrontendConfig } from '../../../platform/config/env'
import BilingualReplyPanel from './BilingualReplyPanel.vue'
import MessageAiReplyPanel from './MessageAiReplyPanel.vue'
import MessageLocalIntelligencePanel from './MessageLocalIntelligencePanel.vue'
import MessageTrustReviewPanel from './MessageTrustReviewPanel.vue'
import type { AiReplyResponse, CommunicationMessageDetailResponse, CommunicationMessageInsight } from '../types/communications'
import type { BilingualReplyFlowResponse } from '../types/bilingualReplyFlow'
import {
  aiSummaryContractFromMetadata,
  communicationExtractionSectionsFromInsight,
  communicationKnowledgeSectionsFromSummaryContract
} from '../helpers/communicationPageModels'

const frontendConfig = loadFrontendConfig()

const props = defineProps<{
  detail: CommunicationMessageDetailResponse | null
  insight: CommunicationMessageInsight | null
}>()

const emit = defineEmits<{
  analyze: []
  reply: []
  createTask: []
  createNote: []
  translate: []
  generateAiReply: [payload: { tone: string; language: string }]
  applyAiReply: [payload: AiReplyResponse]
  reviewSecurity: []
  reviewRecipients: []
  sendBilingualReply: [payload: BilingualReplyFlowResponse]
}>()

const showOriginalFrame = ref(false)
const isBilingualReplyOpen = ref(false)
const shouldLoadRemoteImages = ref(false)

const message = computed(() => props.detail?.message ?? null)
const attachments = computed(() => props.detail?.attachments ?? [])
const messageId = computed(() => message.value?.message_id ?? null)
const summaryContract = computed(() =>
  message.value ? aiSummaryContractFromMetadata(message.value.message_metadata) : null
)
const summarySections = computed(() => {
  const contract = summaryContract.value
  if (!contract) return []
  return [
    { title: 'Key points', items: contract.key_points },
    { title: 'Action items', items: contract.action_items },
    { title: 'Risks', items: contract.risks },
    { title: 'Deadlines', items: contract.deadlines }
  ].filter((section) => section.items.length > 0)
})
const extractionSections = computed(() => communicationExtractionSectionsFromInsight(props.insight))
const knowledgeSections = computed(() => communicationKnowledgeSectionsFromSummaryContract(summaryContract.value))

const bodyText = computed(() => {
  if (!message.value) return ''
  return message.value.body_text ?? ''
})

const renderedBody = computed(() =>
  renderMessageBody({
    bodyHtml: message.value?.body_html,
    bodyText: bodyText.value
  })
)

const isHtmlEmail = computed(() => renderedBody.value.kind === 'html')
const remoteImageUrls = computed(() => {
  const bodyHtml = message.value?.body_html
  return bodyHtml ? remoteImageUrlsFromHtml(bodyHtml) : []
})
const hasRemoteImages = computed(() => remoteImageUrls.value.length > 0)
const displayHtml = computed(() => {
  if (!isHtmlEmail.value) return ''
  if (!hasRemoteImages.value) return renderedBody.value.html
  return rewriteRemoteImageSources(renderedBody.value.html, (url) =>
    shouldLoadRemoteImages.value && messageId.value ? remoteImageProxyUrl(messageId.value, url) : null
  )
})

const originalSrcdoc = computed(() => {
  if (!isHtmlEmail.value) return ''
  const safeHtml = displayHtml.value
  return `<!DOCTYPE html><html><head><base target="_blank"><meta charset="utf-8"><style>
    body { font-family: Arial, Helvetica, sans-serif; color: #1f2933; background: #fff; padding: 1rem; margin: 0; line-height: 1.5; }
    img { max-width: 100%; height: auto; }
    a { color: #2563eb; }
  </style></head><body>${safeHtml}</body></html>`
})

const importanceLabel = computed(() => {
  const score = props.insight?.explain?.reasons?.[0]
  return score ?? null
})

function remoteImageProxyUrl(currentMessageId: string, imageUrl: string): string {
  const base = frontendConfig.apiBaseUrl.replace(/\/+$/, '')
  return `${base}/api/v1/communications/messages/${encodeURIComponent(currentMessageId)}/remote-image?url=${encodeURIComponent(imageUrl)}`
}
</script>

<template>
  <div class="message-body-tab">
    <!-- Render original HTML in sandboxed iframe -->
    <div v-if="isHtmlEmail" class="html-frame-container">
      <div v-if="hasRemoteImages" class="remote-image-notice">
        <div>
          <strong>Remote images blocked</strong>
          <span>{{ remoteImageUrls.length }} external image{{ remoteImageUrls.length === 1 ? '' : 's' }}</span>
        </div>
        <Button
          variant="outline"
          size="sm"
          @click="shouldLoadRemoteImages = !shouldLoadRemoteImages"
        >
          <Icon :icon="shouldLoadRemoteImages ? 'tabler:eye-off' : 'tabler:photo-down'" />
          {{ shouldLoadRemoteImages ? 'Block' : 'Load via proxy' }}
        </Button>
      </div>
      <div v-if="showOriginalFrame" class="original-frame-wrapper">
        <iframe
          class="original-iframe"
          sandbox="allow-popups allow-popups-to-escape-sandbox"
          :srcdoc="originalSrcdoc"
          title="Message body"
        />
        <Button variant="ghost" size="sm" class="toggle-view-btn" @click="showOriginalFrame = false">
          <Icon icon="tabler:code" /> Rendered
        </Button>
      </div>
      <div v-else class="shadow-frame-wrapper">
        <div class="mail-html-body" v-html="displayHtml" />
        <Button variant="ghost" size="sm" class="toggle-view-btn" @click="showOriginalFrame = true">
          <Icon icon="tabler:file-code" /> Original HTML
        </Button>
      </div>
    </div>
    <!-- Plain text fallback -->
    <div v-else class="plain-text-body">
      <pre>{{ bodyText }}</pre>
    </div>

    <!-- Message Intelligence -->
    <div v-if="insight" class="intelligence-card">
      <div class="intel-header">
        <Icon icon="tabler:bulb" class="intel-icon" />
        <span class="intel-title">Message Intelligence</span>
      </div>
      <div v-if="insight.explain?.reasons?.length" class="intel-row">
        <span class="intel-label">Summary:</span>
        <span class="intel-value">{{ insight.explain.reasons.join(', ') }}</span>
      </div>
      <div v-if="insight.language" class="intel-row">
        <span class="intel-label">Language:</span>
        <span class="intel-value">{{ insight.language.language }} ({{ (insight.language.confidence * 100).toFixed(0) }}%)</span>
      </div>
      <div v-if="insight.signature?.has_signature" class="intel-row">
        <span class="intel-label">Signature:</span>
        <span class="intel-value">{{ insight.signature.signature_type }}</span>
      </div>
    </div>

    <MessageAiReplyPanel
      :message-id="messageId"
      :insight="insight"
      @generate-ai-reply="emit('generateAiReply', $event)"
      @apply-ai-reply="emit('applyAiReply', $event)"
    />

    <MessageLocalIntelligencePanel
      :message-id="messageId"
      :insight="insight"
    />

    <MessageTrustReviewPanel
      :message-id="messageId"
      :insight="insight"
      @review-security="emit('reviewSecurity')"
      @review-recipients="emit('reviewRecipients')"
    />

    <section v-if="summarySections.length" class="ai-summary-contract">
      <div class="ai-summary-header">
        <Icon icon="tabler:sparkles" class="intel-icon" />
        <span class="intel-title">AI Summary Review</span>
      </div>
      <div class="ai-summary-grid">
        <div
          v-for="section in summarySections"
          :key="section.title"
          class="ai-summary-section"
        >
          <h4>{{ section.title }}</h4>
          <ul>
            <li v-for="item in section.items" :key="item">{{ item }}</li>
          </ul>
        </div>
      </div>
    </section>

    <section v-if="knowledgeSections.length" class="knowledge-review">
      <div class="knowledge-review-header">
        <Icon icon="tabler:affiliate" class="intel-icon" />
        <span class="intel-title">Knowledge Review</span>
      </div>
      <div class="knowledge-review-grid">
        <article
          v-for="section in knowledgeSections"
          :key="section.kind"
          class="knowledge-review-section"
        >
          <h4>{{ section.title }}</h4>
          <div class="knowledge-review-items">
            <div
              v-for="(item, index) in section.items"
              :key="`${section.kind}-${index}-${item.title}`"
              class="knowledge-review-item"
            >
              <strong>{{ item.title }}</strong>
              <p v-if="item.evidence">{{ item.evidence }}</p>
            </div>
          </div>
        </article>
      </div>
    </section>

    <section v-if="extractionSections.length" class="extraction-review">
      <div class="extraction-review-header">
        <Icon icon="tabler:list-check" class="intel-icon" />
        <span class="intel-title">Extraction Review</span>
      </div>
      <div class="extraction-review-grid">
        <article
          v-for="section in extractionSections"
          :key="section.kind"
          class="extraction-review-section"
        >
          <h4>{{ section.title }}</h4>
          <div class="extraction-review-items">
            <div
              v-for="(item, index) in section.items"
              :key="`${section.kind}-${index}-${item.title}`"
              class="extraction-review-item"
            >
              <strong>{{ item.title }}</strong>
              <div v-if="item.meta.length" class="extraction-review-meta">
                <span v-for="meta in item.meta" :key="meta">{{ meta }}</span>
              </div>
              <p>{{ item.body }}</p>
            </div>
          </div>
        </article>
      </div>
    </section>

    <!-- Workflow action buttons -->
    <div class="workflow-actions">
      <Button variant="outline" size="sm" @click="emit('reply')">
        <Icon icon="tabler:arrow-back-up" /> Reply
      </Button>
      <Button variant="outline" size="sm" @click="emit('createTask')">
        <Icon icon="tabler:checkbox" /> Task
      </Button>
      <Button variant="outline" size="sm" @click="emit('createNote')">
        <Icon icon="tabler:notes" /> Note
      </Button>
      <Button variant="outline" size="sm" @click="emit('translate')">
        <Icon icon="tabler:language" /> Translate
      </Button>
      <Button variant="outline" size="sm" @click="emit('generateAiReply', { tone: 'business', language: 'en' })">
        <Icon icon="tabler:sparkles" /> AI Reply
      </Button>
      <Button variant="outline" size="sm" @click="emit('reviewSecurity')">
        <Icon icon="tabler:shield-search" /> Security
      </Button>
      <Button variant="outline" size="sm" @click="emit('reviewRecipients')">
        <Icon icon="tabler:users-plus" /> Smart CC
      </Button>
      <Button variant="outline" size="sm" @click="isBilingualReplyOpen = !isBilingualReplyOpen">
        <Icon icon="tabler:language-hiragana" /> Bilingual
      </Button>
      <Button variant="outline" size="sm" @click="emit('analyze')">
        <Icon icon="tabler:sparkles" /> Analyze
      </Button>
    </div>

    <BilingualReplyPanel
      v-if="isBilingualReplyOpen && messageId"
      :message-id="messageId"
      @send-bilingual-reply="emit('sendBilingualReply', $event)"
    />
  </div>
</template>

<style scoped>
.message-body-tab {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  height: 100%;
}

.html-frame-container {
  flex: 1;
  min-height: 300px;
  position: relative;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  overflow: hidden;
}

.original-frame-wrapper,
.shadow-frame-wrapper {
  width: 100%;
  height: 100%;
  position: relative;
}

.original-iframe {
  width: 100%;
  height: 100%;
  border: none;
}

.mail-html-body {
  padding: 1rem;
  min-height: 300px;
  color: var(--hh-text-primary, #1f2937);
  background: var(--hh-bg-surface, #fff);
  line-height: 1.5;
  font-size: 0.875rem;
}

.mail-html-body :deep(p) {
  margin: 0 0 0.75em;
}

.mail-html-body :deep(a) {
  color: var(--hh-accent, #2563eb);
}

.mail-html-body :deep(blockquote) {
  margin: 0.5em 0;
  padding-left: 1em;
  border-left: 3px solid var(--hh-border, #d1d5db);
  color: var(--hh-text-secondary, #6b7280);
}

.mail-html-body :deep(img) {
  max-width: 100%;
  height: auto;
}

.mail-html-body :deep(img[data-hermes-remote-src]) {
  min-width: 8rem;
  min-height: 3rem;
  border: 1px dashed var(--hh-border, #d1d5db);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-secondary, #f9fafb) 82%, transparent);
}

.mail-html-body :deep(table) {
  border-collapse: collapse;
  width: 100%;
}

.mail-html-body :deep(td),
.mail-html-body :deep(th) {
  padding: 0.5em;
  border: 1px solid var(--hh-border, #e5e7eb);
}

.toggle-view-btn {
  position: absolute;
  bottom: 0.5rem;
  right: 0.5rem;
  opacity: 0.7;
}
.toggle-view-btn:hover {
  opacity: 1;
}

.plain-text-body {
  flex: 1;
  padding: 1rem;
  background: var(--hh-bg-code, #f9fafb);
  border-radius: 0.375rem;
  overflow: auto;
}
.plain-text-body pre {
  white-space: pre-wrap;
  word-break: break-word;
  font-family: monospace;
  font-size: 0.8125rem;
  margin: 0;
  color: var(--hh-text-primary, #1f2937);
}

.intelligence-card {
  padding: 0.75rem;
  background: var(--hh-bg-info, #f0f9ff);
  border: 1px solid var(--hh-border-info, #bae6fd);
  border-radius: 0.375rem;
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.intel-header,
.ai-summary-header,
.knowledge-review-header,
.extraction-review-header {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.intel-icon {
  width: 16px;
  height: 16px;
  color: var(--hh-accent, #3b82f6);
}

.intel-title {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
}

.intel-row {
  display: flex;
  gap: 0.5rem;
  font-size: 0.75rem;
}

.intel-label {
  color: var(--hh-text-secondary, #6b7280);
  flex-shrink: 0;
}

.intel-value {
  color: var(--hh-text-primary, #1f2937);
}

.ai-summary-contract,
.knowledge-review,
.extraction-review {
  padding: 0.75rem;
  border: 1px solid var(--hh-border-info, #bae6fd);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-info, #f0f9ff) 78%, transparent);
}

.ai-summary-grid,
.knowledge-review-grid,
.extraction-review-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(11rem, 1fr));
  gap: 0.625rem;
  margin-top: 0.625rem;
}

.ai-summary-section,
.knowledge-review-section,
.extraction-review-section {
  min-width: 0;
  padding: 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 82%, transparent);
}

.ai-summary-section h4,
.knowledge-review-section h4,
.extraction-review-section h4 {
  margin: 0 0 0.375rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.ai-summary-section ul {
  display: grid;
  gap: 0.375rem;
  margin: 0;
  padding-left: 1rem;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
  line-height: 1.4;
}

.extraction-review-items {
  display: grid;
  gap: 0.5rem;
}

.knowledge-review-items {
  display: grid;
  gap: 0.5rem;
}

.knowledge-review-item {
  display: grid;
  gap: 0.25rem;
  min-width: 0;
}

.knowledge-review-item strong {
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
}

.knowledge-review-item p {
  margin: 0;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
  line-height: 1.4;
}

.extraction-review-item {
  display: grid;
  gap: 0.25rem;
  min-width: 0;
}

.extraction-review-item strong {
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
}

.extraction-review-item p {
  margin: 0;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
  line-height: 1.4;
}

.extraction-review-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.extraction-review-meta span {
  padding: 0.125rem 0.375rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 999px;
  color: var(--hh-text-secondary, #6b7280);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 72%, transparent);
  font-size: 0.6875rem;
}

.workflow-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  padding: 0.5rem 0;
}
</style>
