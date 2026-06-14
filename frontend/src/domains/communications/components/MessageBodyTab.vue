<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import { renderMessageBody } from '../../../shared/sanitize/emailHtml'
import type { MailMessageDetailResponse, MailMessageInsight } from '../types/communications'

const props = defineProps<{
  detail: MailMessageDetailResponse | null
  insight: MailMessageInsight | null
}>()

const emit = defineEmits<{
  analyze: []
  reply: []
  createTask: []
  createNote: []
  translate: []
}>()

const showOriginalFrame = ref(false)

const message = computed(() => props.detail?.message ?? null)
const attachments = computed(() => props.detail?.attachments ?? [])

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

const originalSrcdoc = computed(() => {
  if (!isHtmlEmail.value) return ''
  const safeHtml = renderedBody.value.html
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
</script>

<template>
  <div class="message-body-tab">
    <!-- Render original HTML in sandboxed iframe -->
    <div v-if="isHtmlEmail" class="html-frame-container">
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
        <div class="mail-html-body" v-html="renderedBody.html" />
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
      <Button variant="outline" size="sm" @click="emit('analyze')">
        <Icon icon="tabler:sparkles" /> Analyze
      </Button>
    </div>
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

.intel-header {
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

.workflow-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  padding: 0.5rem 0;
}
</style>
