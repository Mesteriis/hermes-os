import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { PersonItem, PersonDossier, PersonIdentityCandidate, PersonIdentity, Relationship } from '../types/persona'
import { reviewIdentityCandidate, assignIdentityTrace, reviewRelationship } from '../api/personas'

export function formatIdentityTraceKind(kind: string): string {
  const labels: Record<string, string> = {
    email: 'Email',
    phone: 'Phone',
    telegram: 'Telegram',
    whatsapp: 'WhatsApp',
    social: 'Social Profile',
    name: 'Name',
    organization: 'Organization'
  }
  return labels[kind] || kind
}

export function formatIdentityTraceValue(trace: PersonIdentity): string {
  return trace.value || trace.identity_type
}

export function identityTraceConfidence(trace: PersonIdentity): string {
  return `${Math.round(trace.confidence * 100)}%`
}

export function formatRelationshipType(type: string): string {
  const labels: Record<string, string> = {
    colleague: 'Colleague',
    manager: 'Manager',
    client: 'Client',
    partner: 'Partner',
    vendor: 'Vendor',
    friend: 'Friend',
    family: 'Family',
    acquaintance: 'Acquaintance',
    competitor: 'Competitor',
    other: 'Other'
  }
  return labels[type] || type
}

export function formatRelationshipScore(score: number): string {
  if (score >= 0.8) return 'Strong'
  if (score >= 0.5) return 'Moderate'
  return 'Weak'
}

export function formatRelationshipEndpoint(kind: string, id: string): string {
  return `${kind}:${id.slice(0, 8)}...`
}

export function personIdentityPairKey(leftPersonId: string, rightPersonId: string): string {
  return leftPersonId <= rightPersonId
    ? `${leftPersonId}:${rightPersonId}`
    : `${rightPersonId}:${leftPersonId}`
}

export function dossierSectionPreview(dossier: PersonDossier): string[] {
  const words = (dossier.summary || '').split(/\s+/).filter(Boolean)
  return [...new Set(words.slice(0, 10))]
}

export const usePersonasStore = defineStore('personas', () => {
  const selectedPersonIndex = ref(0)
  const loadedDossierPersonId = ref<string | null>(null)
  const personDossier = ref<PersonDossier | null>(null)
  const personDossierError = ref('')
  const isPersonDossierLoading = ref(false)
  const identityCandidatesError = ref('')
  const identityTracesError = ref('')
  const relationshipsError = ref('')
  const assigningIdentityTraceId = ref<string | null>(null)
  const reviewingRelationshipId = ref<string | null>(null)

  const identityCandidates = ref<PersonIdentityCandidate[]>([])
  const identityTraces = ref<PersonIdentity[]>([])
  const relationships = ref<Relationship[]>([])

  const suggestedIdentityCandidates = computed(() =>
    identityCandidates.value.filter((item) => item.review_state === 'suggested')
  )

  function setIdentityCandidates(items: PersonIdentityCandidate[]) {
    identityCandidates.value = items
  }

  function setIdentityTraces(items: PersonIdentity[]) {
    identityTraces.value = items
  }

  function setRelationships(items: Relationship[]) {
    relationships.value = items
  }

  function selectPerson(index: number) {
    selectedPersonIndex.value = index
  }

  function setPersonDossier(dossier: PersonDossier | null, error: string) {
    personDossier.value = dossier
    personDossierError.value = error
  }

  function setPersonDossierLoading(loading: boolean) {
    isPersonDossierLoading.value = loading
  }

  function setLoadedDossierPersonId(id: string | null) {
    loadedDossierPersonId.value = id
  }

  async function reviewCandidate(candidate: PersonIdentityCandidate, state: PersonIdentityCandidate['review_state']) {
    try {
      await reviewIdentityCandidate(candidate.candidate_id, state as any)
    } catch (e: any) {
      identityCandidatesError.value = e.message || 'Review failed'
    }
  }

  async function assignTraceToPersona(trace: PersonIdentity, personId: string) {
    assigningIdentityTraceId.value = trace.id
    try {
      await assignIdentityTrace(trace.id, personId)
    } catch (e: any) {
      identityTracesError.value = e.message || 'Assignment failed'
    }
    assigningIdentityTraceId.value = null
  }

  async function reviewRelation(relationship: Relationship, reviewState: string) {
    reviewingRelationshipId.value = relationship.relationship_id
    try {
      await reviewRelationship(relationship.relationship_id, reviewState)
    } catch (e: any) {
      relationshipsError.value = e.message || 'Review failed'
    }
    reviewingRelationshipId.value = null
  }

  return {
    selectedPersonIndex,
    personDossier,
    personDossierError,
    isPersonDossierLoading,
    identityCandidatesError,
    identityTracesError,
    relationshipsError,
    assigningIdentityTraceId,
    reviewingRelationshipId,
    suggestedIdentityCandidates,
    identityCandidates,
    identityTraces,
    relationships,
    loadedDossierPersonId,
    setIdentityCandidates,
    setIdentityTraces,
    setRelationships,
    selectPerson,
    setPersonDossier,
    setPersonDossierLoading,
    setLoadedDossierPersonId,
    reviewCandidate,
    assignTraceToPersona,
    reviewRelation
  }
})
