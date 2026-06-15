<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import {
  useDetectMessageLanguageMutation,
  useExplainMessageMutation
} from '../queries/useCommunicationsQuery'
import type {
  LanguageDetection,
  MailMessageInsight,
  MessageExplainResponse
} from '../types/communications'

const props = defineProps<{
  messageId: string | null
  insight: MailMessageInsight | null
}>()

const explainMutation = useExplainMessageMutation()
const languageMutation = useDetectMessageLanguageMutation()
const explainResult = ref<MessageExplainResponse | null>(null)
const languageResult = ref<LanguageDetection | null>(null)
const errorMessage = ref('')

const currentExplain = computed(() => explainResult.value ?? props.insight?.explain ?? null)
const currentLanguage = computed(() => languageResult.value ?? props.insight?.language ?? null)
const isRunning = computed(() => explainMutation.isPending.value || languageMutation.isPending.value)

watch(
  () => props.messageId,
  () => {
    explainResult.value = null
    languageResult.value = null
    errorMessage.value = ''
  }
)

async function explainMessage(): Promise<void> {
  if (!props.messageId) return
  errorMessage.value = ''
  try {
    explainResult.value = await explainMutation.mutateAsync(props.messageId)
  } catch (e) {
    errorMessage.value = e instanceof Error ? e.message : 'Importance explanation failed'
  }
}

async function detectLanguage(): Promise<void> {
  if (!props.messageId) return
  errorMessage.value = ''
  try {
    languageResult.value = await languageMutation.mutateAsync(props.messageId)
  } catch (e) {
    errorMessage.value = e instanceof Error ? e.message : 'Language detection failed'
  }
}
</script>

<template>
  <section class="local-intelligence-panel">
    <div class="local-intelligence-header">
      <div class="local-intelligence-title">
        <Icon icon="tabler:brain" class="intel-icon" />
        <span>Importance & Language</span>
      </div>
      <div class="local-intelligence-actions">
        <Button variant="outline" size="sm" :disabled="!messageId" :loading="explainMutation.isPending.value" @click="explainMessage">
          <Icon icon="tabler:info-circle" /> Why this matters
        </Button>
        <Button variant="outline" size="sm" :disabled="!messageId" :loading="languageMutation.isPending.value" @click="detectLanguage">
          <Icon icon="tabler:language" /> Detect language
        </Button>
      </div>
    </div>

    <div v-if="currentExplain" class="local-intelligence-card">
      <strong>Importance</strong>
      <ul>
        <li v-for="reason in currentExplain.reasons" :key="reason">{{ reason }}</li>
      </ul>
    </div>

    <div v-if="currentLanguage" class="local-intelligence-card">
      <strong>Language</strong>
      <span>{{ currentLanguage.language }} · {{ (currentLanguage.confidence * 100).toFixed(0) }}%</span>
    </div>

    <p v-if="!currentExplain && !currentLanguage && !isRunning" class="local-intelligence-empty">
      No local intelligence review has been run for this message.
    </p>
    <p v-if="errorMessage" class="local-intelligence-error">{{ errorMessage }}</p>
  </section>
</template>

<style scoped>
.local-intelligence-panel {
  display: grid;
  gap: 0.625rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 82%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.local-intelligence-header,
.local-intelligence-title,
.local-intelligence-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.local-intelligence-header {
  justify-content: space-between;
}

.local-intelligence-title {
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
  font-weight: 700;
}

.intel-icon {
  width: 16px;
  height: 16px;
  color: var(--hh-accent, #2563eb);
}

.local-intelligence-card {
  display: grid;
  gap: 0.25rem;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
}

.local-intelligence-card ul {
  margin: 0;
  padding-left: 1rem;
}

.local-intelligence-empty {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.local-intelligence-error {
  color: var(--hh-text-error, #ef4444);
  font-size: 0.75rem;
}
</style>
