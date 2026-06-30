import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import {
  useIdentityCandidatesQuery,
  useIdentityTracesQuery,
  usePersonsQuery,
  useRelationshipsQuery
} from './usePersonasQuery'
import { usePersonasStore } from '../stores/personas'
import type { PersonIdentityCandidate, PersonItem } from '../types/persona'

export function usePersonsPageSurface() {
  const { t } = useI18n()
  const store = usePersonasStore()

  const { data: personsData } = usePersonsQuery()
  const { data: identityCandidatesData } = useIdentityCandidatesQuery()
  const { data: identityTracesData } = useIdentityTracesQuery()
  const { data: relationshipsData } = useRelationshipsQuery()

  const personList = computed<PersonItem[]>(() =>
    (personsData.value ?? []).map((person) => ({
      person_id: person.person_id,
      name: person.display_name,
      role: person.preferred_channel || t('Contact'),
      company: person.email_address,
      status: person.last_interaction_at ? t('Online') : undefined,
      channel: person.preferred_channel ?? undefined
    }))
  )

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

  const selectedPersonaId = computed(() => selectedPerson.value?.person_id ?? null)
  const identityTraces = computed(() => identityTracesData.value ?? [])
  const relationships = computed(() => relationshipsData.value ?? [])

  function identityConfidence(item: PersonIdentityCandidate): string {
    return `${Math.round(item.confidence * 100)}%`
  }

  async function setIdentityCandidateReview(candidate: PersonIdentityCandidate, state: string) {
    await store.reviewCandidate(candidate, state as PersonIdentityCandidate['review_state'])
  }

  async function splitConfirmedIdentityMerge(candidate: PersonIdentityCandidate) {
    await store.reviewCandidate(candidate, 'suggested')
  }

  function splitCandidateForConfirmedMerge(): PersonIdentityCandidate | null {
    return null
  }

  return {
    confirmedMergeIdentityCandidates,
    identityConfidence,
    identityTraces,
    personList,
    relationships,
    selectedPerson,
    selectedPersonaId,
    setIdentityCandidateReview,
    splitCandidateForConfirmedMerge,
    splitConfirmedIdentityMerge,
    store,
    suggestedIdentityCandidates
  }
}
