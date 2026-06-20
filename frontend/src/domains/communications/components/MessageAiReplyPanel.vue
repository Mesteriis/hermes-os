<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type { AiReplyResponse, CommunicationMessageInsight } from '../types/communications'
import { useGenerateAiReplyVariantsMutation } from '../queries/useCommunicationsQuery'

const props = defineProps<{
  messageId: string | null
  insight: CommunicationMessageInsight | null
}>()

const emit = defineEmits<{
  generateAiReply: [payload: { tone: string; language: string }]
  applyAiReply: [payload: AiReplyResponse]
}>()

const selectedAiReplyTone = ref('business')
const selectedAiReplyLanguage = ref('en')

const aiReplyToneOptions = ['formal', 'business', 'friendly', 'short', 'detailed']
const aiReplyLanguageOptions = [
  { value: 'en', label: 'English' },
  { value: 'ru', label: 'Russian' }
]

const aiReply = computed(() => props.insight?.aiReply ?? null)
const generateAiReplyVariantsMutation = useGenerateAiReplyVariantsMutation()
const replyVariants = ref<AiReplyResponse[]>([])
const variantsPending = computed(() => generateAiReplyVariantsMutation.isPending.value)
const variantsError = computed(() => generateAiReplyVariantsMutation.error.value?.message ?? '')

async function generateVariants() {
  if (!props.messageId) return
  try {
    const result = await generateAiReplyVariantsMutation.mutateAsync({
      messageId: props.messageId,
      languages: aiReplyLanguageOptions.map((language) => language.value),
      tones: aiReplyToneOptions
    })
    replyVariants.value = result.variants
  } catch {
    replyVariants.value = []
  }
}
</script>

<template>
  <section class="ai-reply-review">
    <div class="ai-reply-header">
      <div class="ai-reply-title">
        <Icon icon="tabler:sparkles" class="intel-icon" />
        <span class="intel-title">AI Reply Review</span>
      </div>
      <div class="ai-reply-controls">
        <label>
          <span>Tone</span>
          <select v-model="selectedAiReplyTone">
            <option v-for="tone in aiReplyToneOptions" :key="tone" :value="tone">
              {{ tone }}
            </option>
          </select>
        </label>
        <label>
          <span>Language</span>
          <select v-model="selectedAiReplyLanguage">
            <option v-for="language in aiReplyLanguageOptions" :key="language.value" :value="language.value">
              {{ language.label }}
            </option>
          </select>
        </label>
        <Button
          variant="outline"
          size="sm"
          :disabled="!messageId"
          @click="emit('generateAiReply', { tone: selectedAiReplyTone, language: selectedAiReplyLanguage })"
        >
          <Icon icon="tabler:sparkles" /> Generate
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :disabled="!messageId || variantsPending"
          @click="generateVariants"
        >
          <Icon icon="tabler:versions" /> Variants
        </Button>
      </div>
    </div>
    <p v-if="variantsError" class="ai-reply-error">{{ variantsError }}</p>
    <div v-if="aiReply" class="ai-reply-card">
      <div class="ai-reply-meta">
        <span>{{ aiReply.tone || selectedAiReplyTone }}</span>
        <span>{{ aiReply.language || selectedAiReplyLanguage }}</span>
        <span v-if="aiReply.generated === false">{{ aiReply.reason || 'Not generated' }}</span>
      </div>
      <strong>{{ aiReply.subject || 'Reply draft' }}</strong>
      <p>{{ aiReply.body || aiReply.reason || 'No reply body returned.' }}</p>
      <Button
        variant="ghost"
        size="sm"
        :disabled="!aiReply.body"
        @click="emit('applyAiReply', aiReply)"
      >
        <Icon icon="tabler:pencil" /> Apply to compose
      </Button>
    </div>
    <div v-if="replyVariants.length" class="ai-reply-variants">
      <article
        v-for="(variant, index) in replyVariants"
        :key="`${variant.language}-${variant.tone}-${index}`"
        class="ai-reply-card"
      >
        <div class="ai-reply-meta">
          <span>{{ variant.tone || 'tone' }}</span>
          <span>{{ variant.language || 'language' }}</span>
        </div>
        <strong>{{ variant.subject || 'Reply variant' }}</strong>
        <p>{{ variant.body || 'No reply body returned.' }}</p>
        <Button
          variant="ghost"
          size="sm"
          :disabled="!variant.body"
          @click="emit('applyAiReply', variant)"
        >
          <Icon icon="tabler:pencil" /> Apply
        </Button>
      </article>
    </div>
  </section>
</template>

<style scoped>
.ai-reply-review {
  display: grid;
  gap: 0.625rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border-info, #bae6fd);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-info, #f0f9ff) 78%, transparent);
}

.ai-reply-header,
.ai-reply-title,
.ai-reply-controls {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.ai-reply-header {
  justify-content: space-between;
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

.ai-reply-controls label {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.ai-reply-controls select {
  min-height: 1.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 86%, transparent);
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
}

.ai-reply-card {
  display: grid;
  gap: 0.375rem;
  min-width: 0;
  padding: 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 82%, transparent);
}

.ai-reply-variants {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
  gap: 0.5rem;
}

.ai-reply-card strong {
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
}

.ai-reply-card p {
  margin: 0;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.8125rem;
  line-height: 1.4;
  white-space: pre-wrap;
}

.ai-reply-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.ai-reply-meta span {
  padding: 0.125rem 0.375rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 999px;
  color: var(--hh-text-secondary, #6b7280);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 72%, transparent);
  font-size: 0.6875rem;
}

.ai-reply-error {
  margin: 0;
  color: var(--hh-danger, #b91c1c);
  font-size: 0.75rem;
}
</style>
