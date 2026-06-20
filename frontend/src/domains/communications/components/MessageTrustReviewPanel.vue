<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type { CommunicationMessageInsight } from '../types/communications'

const props = defineProps<{
  messageId: string | null
  insight: CommunicationMessageInsight | null
}>()

const emit = defineEmits<{
  reviewSecurity: []
  reviewRecipients: []
}>()

const smartCc = computed(() => props.insight?.smartCc ?? null)
const authReview = computed(() => props.insight?.auth ?? null)
const signatureReview = computed(() => props.insight?.signature ?? null)
const authRisk = computed(() => authReview.value?.risk ?? null)
const authChecks = computed(() => {
  const auth = authReview.value
  if (!auth) return []
  return [
    { label: 'SPF', result: auth.auth.spf?.result ?? 'missing', passed: auth.risk.spf_pass },
    { label: 'DKIM', result: auth.auth.dkim?.result ?? 'missing', passed: auth.risk.dkim_pass },
    { label: 'DMARC', result: auth.auth.dmarc?.result ?? 'missing', passed: auth.risk.dmarc_pass }
  ]
})
</script>

<template>
  <section class="message-review-grid">
    <article class="security-review">
      <div class="review-header">
        <Icon icon="tabler:shield-check" class="intel-icon" />
        <span class="intel-title">Security Review</span>
      </div>
      <Button variant="outline" size="sm" :disabled="!messageId" @click="emit('reviewSecurity')">
        <Icon icon="tabler:shield-search" /> Check auth
      </Button>
      <div v-if="authRisk" class="security-risk" :class="{ risky: authRisk.is_spoofed }">
        <strong>{{ authRisk.risk_summary }}</strong>
        <div class="auth-chip-row">
          <span
            v-for="check in authChecks"
            :key="check.label"
            class="auth-chip"
            :class="{ passed: check.passed }"
          >
            {{ check.label }} {{ check.result }}
          </span>
        </div>
      </div>
      <div v-if="signatureReview" class="signature-review">
        <span>{{ signatureReview.has_signature ? 'Signed message' : 'No signature detected' }}</span>
        <span v-if="signatureReview.signature_type">{{ signatureReview.signature_type }}</span>
        <span v-if="signatureReview.cert_expiry_warning">{{ signatureReview.cert_expiry_warning }}</span>
      </div>
    </article>

    <article class="recipient-review">
      <div class="review-header">
        <Icon icon="tabler:user-plus" class="intel-icon" />
        <span class="intel-title">Recipient Suggestions</span>
      </div>
      <Button variant="outline" size="sm" :disabled="!messageId" @click="emit('reviewRecipients')">
        <Icon icon="tabler:users-plus" /> Smart CC
      </Button>
      <div v-if="smartCc" class="recipient-chip-row">
        <span v-if="smartCc.suggestions.length === 0" class="empty-review">No suggestions</span>
        <span v-for="suggestion in smartCc.suggestions" :key="suggestion" class="recipient-chip">
          {{ suggestion }}
        </span>
      </div>
    </article>
  </section>
</template>

<style scoped>
.message-review-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
  gap: 0.625rem;
}

.security-review,
.recipient-review {
  display: grid;
  align-content: start;
  gap: 0.625rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border-info, #bae6fd);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-info, #f0f9ff) 78%, transparent);
}

.review-header {
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

.security-risk,
.signature-review,
.recipient-chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.security-risk {
  display: grid;
}

.security-risk strong {
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
}

.security-risk.risky strong {
  color: var(--hh-danger, #dc2626);
}

.auth-chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.auth-chip,
.recipient-chip,
.empty-review {
  padding: 0.125rem 0.375rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 999px;
  color: var(--hh-text-secondary, #6b7280);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 72%, transparent);
  font-size: 0.6875rem;
}

.auth-chip.passed {
  color: var(--hh-success, #15803d);
}
</style>
