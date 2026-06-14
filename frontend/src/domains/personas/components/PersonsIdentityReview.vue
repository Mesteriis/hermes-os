<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { PersonIdentityCandidate } from '../types/persona'

const { t } = useI18n()

defineProps<{
  suggestedIdentityCandidates: PersonIdentityCandidate[]
  confirmedMergeIdentityCandidates: PersonIdentityCandidate[]
  isIdentityCandidatesLoading: boolean
  identityCandidatesError: string
  identityConfidence: (candidate: PersonIdentityCandidate) => string
  setIdentityCandidateReview: (candidate: PersonIdentityCandidate, state: string) => Promise<void>
  splitConfirmedIdentityMerge: (candidate: PersonIdentityCandidate) => Promise<void>
  splitCandidateForConfirmedMerge: (candidate: PersonIdentityCandidate) => PersonIdentityCandidate | null
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="persons-identity-review">
    <section class="panel info-card">
      <h2>{{ t('Person Identity Review') }}</h2>
      <p class="identity-note">{{ t('Person merges are only suggested and are not applied until confirmed.') }}</p>
      <p v-if="isIdentityCandidatesLoading" class="inline-copy">{{ t('Loading identity suggestions…') }}</p>
      <p v-else-if="identityCandidatesError" class="inline-error">{{ identityCandidatesError }}</p>
      <p v-else-if="suggestedIdentityCandidates.length === 0 && confirmedMergeIdentityCandidates.length === 0" class="inline-copy">
        {{ t('No identity suggestions right now.') }}
      </p>
      <template v-else>
        <div v-for="candidate in suggestedIdentityCandidates" :key="candidate.candidate_id" class="identity-candidate-row">
          <div>
            <strong>{{ candidate.candidate_kind }}</strong>
            <p>{{ candidate.evidence_summary }}</p>
            <small>Left: {{ candidate.left_person_id }}</small>
            <small>Right: {{ candidate.right_person_id ?? t('N/A') }}</small>
            <small>{{ t('Confidence') }}: {{ identityConfidence(candidate) }} · {{ candidate.review_state }}</small>
          </div>
          <div class="identity-actions">
            <button type="button" @click="() => setIdentityCandidateReview(candidate, 'user_confirmed')">
              <Icon icon="tabler:check" :size="15" /> {{ t('Confirm') }}
            </button>
            <button type="button" @click="() => setIdentityCandidateReview(candidate, 'user_rejected')">
              <Icon icon="tabler:x" :size="15" /> {{ t('Reject') }}
            </button>
          </div>
        </div>
        <div v-for="candidate in confirmedMergeIdentityCandidates" :key="candidate.candidate_id" class="identity-candidate-row">
          <div>
            <strong>{{ candidate.candidate_kind }}</strong>
            <p>{{ candidate.evidence_summary }}</p>
            <small>Left: {{ candidate.left_person_id }}</small>
            <small>Right: {{ candidate.right_person_id ?? t('N/A') }}</small>
            <small>{{ t('Confidence') }}: {{ identityConfidence(candidate) }} · {{ candidate.review_state }}</small>
          </div>
          <div class="identity-actions">
            <button
              type="button"
              :disabled="splitCandidateForConfirmedMerge(candidate) === null"
              :title="splitCandidateForConfirmedMerge(candidate) === null ? t('Refresh identity candidates to create a split review for this confirmed link') : undefined"
              @click="() => splitConfirmedIdentityMerge(candidate)"
            >
              <Icon icon="tabler:arrows-split" :size="15" /> {{ t('Split') }}
            </button>
          </div>
        </div>
      </template>
    </section>
  </div>
</template>

<style scoped>
.identity-note {
  margin: 0 0 8px;
  color: var(--hh-color-text-muted);
  font-size: 12px;
}
.identity-candidate-row {
  display: grid;
  gap: 8px;
  padding: 10px 0;
  border-top: 1px solid rgba(102, 189, 180, 0.08);
}
.identity-candidate-row:first-child {
  border-top: none;
}
.identity-candidate-row strong {
  display: block;
  margin-bottom: 3px;
}
.identity-candidate-row p {
  margin: 0 0 4px;
  color: #dbe9e8;
}
.identity-candidate-row small {
  display: block;
  color: var(--hh-color-text-muted);
}
.identity-actions {
  display: inline-flex;
  gap: 7px;
  margin-top: 8px;
}
.identity-actions button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  height: auto;
  font-size: 11px;
}
</style>
