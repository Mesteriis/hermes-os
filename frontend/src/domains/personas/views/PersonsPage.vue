<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { usePersonsQuery, useIdentityCandidatesQuery, useIdentityTracesQuery, useRelationshipsQuery } from '../queries/usePersonasQuery'
import { usePersonasStore } from '../stores/personas'
import PersonsList from '../components/PersonsList.vue'
import PersonsDetail from '../components/PersonsDetail.vue'
import PersonsIdentityReview from '../components/PersonsIdentityReview.vue'
import PersonsIdentityTraceReview from '../components/PersonsIdentityTraceReview.vue'
import PersonsRelationshipReview from '../components/PersonsRelationshipReview.vue'
import type { PersonItem, PersonIdentityCandidate } from '../types/persona'

const { t } = useI18n()
const store = usePersonasStore()

const { data: personsData } = usePersonsQuery()
const { data: identityCandidatesData } = useIdentityCandidatesQuery()
const { data: identityTracesData } = useIdentityTracesQuery()
const { data: relationshipsData } = useRelationshipsQuery()

const personList = computed<PersonItem[]>(() => {
  return (personsData.value ?? []).map((p) => ({
    person_id: p.person_id,
    name: p.display_name,
    role: p.preferred_channel || t('Contact'),
    company: p.email_address,
    status: p.last_interaction_at ? t('Online') : undefined,
    channel: p.preferred_channel ?? undefined
  }))
})

const selectedPerson = computed(() =>
  personList.value[store.selectedPersonIndex] ?? personList.value[0] ?? null
)

const suggestedIdentityCandidates = computed(() =>
  (identityCandidatesData.value ?? []).filter(
    (item: PersonIdentityCandidate) => item.review_state === 'suggested'
  )
)

const confirmedMergeIdentityCandidates = computed(() =>
  (identityCandidatesData.value ?? []).filter(
    (item: PersonIdentityCandidate) =>
      item.candidate_kind === 'merge_persons' &&
      item.review_state === 'user_confirmed'
  )
)

function identityConfidence(item: PersonIdentityCandidate): string {
  return `${Math.round(item.confidence * 100)}%`
}

async function setIdentityCandidateReview(candidate: PersonIdentityCandidate, state: string) {
  await store.reviewCandidate(candidate, state as any)
}

async function splitConfirmedIdentityMerge(candidate: PersonIdentityCandidate) {
  // Simplified: just mark back to suggested to re-evaluate
  await store.reviewCandidate(candidate, 'suggested' as any)
}

function splitCandidateForConfirmedMerge(candidate: PersonIdentityCandidate): PersonIdentityCandidate | null {
  return null
}

async function loadRelationships() {
  // Relationships loaded via TanStack Query
}

async function loadTraces() {
  // Traces loaded via TanStack Query
}
</script>

<template>
  <section class="persons-page">
    <div class="persons-layout">
      <PersonsList
        :personList="personList"
        :selectedPersonIndex="store.selectedPersonIndex"
        @selectPerson="store.selectPerson"
      />
      <PersonsDetail
        :selectedPerson="selectedPerson"
        :personDossier="store.personDossier"
        :isPersonDossierLoading="store.isPersonDossierLoading"
        :personDossierError="store.personDossierError"
        :whatsNew="[]"
        :projects="[]"
      />
      <aside class="stacked-rail">
        <div class="widget-frame" data-widget-id="persons-ai-summary">
          <section class="panel info-card">
            <h2>{{ t('AI Summary') }}</h2>
            <p>{{ t('John is a key strategic partner and decision maker. You have a strong professional relationship with frequent communication across multiple projects.') }}</p>
          </section>
        </div>
        <PersonsIdentityReview
          :suggestedIdentityCandidates="suggestedIdentityCandidates"
          :confirmedMergeIdentityCandidates="confirmedMergeIdentityCandidates"
          :isIdentityCandidatesLoading="false"
          :identityCandidatesError="store.identityCandidatesError"
          :identityConfidence="identityConfidence"
          :setIdentityCandidateReview="setIdentityCandidateReview"
          :splitConfirmedIdentityMerge="splitConfirmedIdentityMerge"
          :splitCandidateForConfirmedMerge="splitCandidateForConfirmedMerge"
        />
        <PersonsIdentityTraceReview
          :identityTraces="identityTracesData ?? []"
          :persons="personList"
          :selectedPersonaId="selectedPerson?.person_id ?? null"
          :isLoading="false"
          :error="store.identityTracesError"
          :assigningIdentityTraceId="store.assigningIdentityTraceId"
          :onReload="loadTraces"
          :onAssign="store.assignTraceToPersona"
        />
        <PersonsRelationshipReview
          :relationships="relationshipsData ?? []"
          :selectedPersonaId="selectedPerson?.person_id ?? null"
          :isLoading="false"
          :error="store.relationshipsError"
          :reviewingRelationshipId="store.reviewingRelationshipId"
          :onReload="loadRelationships"
          :onReview="store.reviewRelation"
        />
        <div class="widget-frame" data-widget-id="persons-related-documents">
          <section class="panel info-card">
            <h2>{{ t('Related Documents') }}</h2>
            <p>{{ t('Documents will appear here when processing is complete.') }}</p>
          </section>
        </div>
        <div class="widget-frame" data-widget-id="persons-recent-notes">
          <section class="panel info-card">
            <h2>{{ t('Recent Notes') }}</h2>
            <p>{{ t('Discussed expansion to EU market') }}</p>
            <p>{{ t('Prefers email for official communication') }}</p>
            <p>{{ t('Interested in AI/ML integration') }}</p>
          </section>
        </div>
      </aside>
    </div>
  </section>
</template>

<style scoped>
.persons-page {
  display: grid;
  grid-template-columns: repeat(var(--hh-layout-columns), minmax(0, 1fr));
  grid-auto-flow: row;
  grid-auto-rows: min-content;
  align-content: start;
  gap: var(--hh-layout-gap);
  height: 100%;
  min-height: 0;
  overflow: hidden;
  padding-right: 0;
}
.persons-page > * {
  grid-column: 1 / -1;
  min-width: 0;
}
.persons-layout {
  --hh-zone-rows: 12;
  display: grid;
  grid-template-columns: repeat(var(--hh-layout-columns), minmax(0, 1fr));
  grid-auto-flow: dense;
  grid-auto-rows: min-content;
  align-content: start;
  align-items: stretch;
  gap: var(--hh-layout-gap);
  width: 100%;
  min-width: 0;
  min-height: 0;
  max-height: 100%;
  overflow: hidden;
}
</style>
