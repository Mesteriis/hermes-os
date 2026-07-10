import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import {
  useAssignIdentityTraceMutation,
  useIdentityCandidatesQuery,
  useIdentityTracesQuery,
  useOwnerPersonaQuery,
  usePersonasQuery,
  useRelationshipsQuery,
  useReviewIdentityCandidateMutation,
  useSetOwnerPersonaMutation,
  useUpdatePersonaAddressBookMembershipMutation
} from './usePersonasQuery'
import { usePersonasStore } from '../stores/personas'
import type {
  EnrichedPersona,
  OwnerPersona,
  PersonaDirectoryFilter,
  PersonaPanelProfile,
  PersonaIdentity,
  PersonaIdentityCandidate,
  PersonaIdentityReviewState,
  PersonaItem,
  PersonaWorkspaceSection
} from '../types/persona'

export function usePersonasPageSurface() {
  const { t } = useI18n()
  const store = usePersonasStore()
  const personaSearchQuery = ref('')
  const directoryFilter = ref<PersonaDirectoryFilter>('all')
  const activeSection = ref<PersonaWorkspaceSection>('overview')

  const personasQuery = usePersonasQuery()
  const ownerPersonaQuery = useOwnerPersonaQuery()
  const identityCandidatesQuery = useIdentityCandidatesQuery()
  const identityTracesQuery = useIdentityTracesQuery()
  const setOwnerPersonaMutation = useSetOwnerPersonaMutation()
  const updatePersonaAddressBookMembershipMutation = useUpdatePersonaAddressBookMembershipMutation()
  const reviewIdentityCandidateMutation = useReviewIdentityCandidateMutation()
  const assignIdentityTraceMutation = useAssignIdentityTraceMutation()

  const personas = computed(() => personasQuery.data.value ?? [])
  const ownerPersona = computed<PersonaPanelProfile | null>(() =>
    ownerProfile(ownerPersonaQuery.data.value, personas.value)
  )
  const filteredPersonas = computed(() => {
    const query = personaSearchQuery.value.trim().toLowerCase()
    if (!query) return personas.value

    return personas.value.filter((persona) => {
      if (directoryFilter.value === 'address_book' && !persona.is_address_book) {
        return false
      }

      return [
        persona.display_name,
        persona.email_address,
        persona.language,
        persona.preferred_channel,
        persona.tone,
        persona.writing_style
      ].some((value) => value?.toLowerCase().includes(query))
    })
  })

  const personaList = computed<PersonaItem[]>(() =>
    filteredPersonas.value.map((persona) => ({
      persona_id: persona.persona_id,
      name: persona.display_name,
      role: persona.preferred_channel || t('Persona'),
      company: persona.email_address || t('No email'),
      status: persona.last_interaction_at ? t('Active') : undefined,
      channel: persona.preferred_channel ?? undefined
    }))
  )

  const selectedPersona = computed<PersonaPanelProfile | null>(() => {
    const selected = filteredPersonas.value[store.selectedPersonaIndex] ?? filteredPersonas.value[0]
    if (!selected) return null

    return {
      ...selected,
      is_owner: ownerPersona.value?.persona_id === selected.persona_id
    }
  })

  const selectedPersonaId = computed(() => selectedPersona.value?.persona_id ?? null)
  const relationshipsQuery = useRelationshipsQuery(selectedPersonaId)
  const identityTraces = computed(() => identityTracesQuery.data.value ?? [])
  const relationships = computed(() => relationshipsQuery.data.value ?? [])
  const suggestedIdentityCandidates = computed(() =>
    (identityCandidatesQuery.data.value ?? []).filter(
      (item: PersonaIdentityCandidate) => item.review_state === 'suggested'
    )
  )
  const confirmedMergeIdentityCandidates = computed(() =>
    (identityCandidatesQuery.data.value ?? []).filter(
      (item: PersonaIdentityCandidate) =>
        item.candidate_kind === 'merge_personas' && item.review_state === 'user_confirmed'
    )
  )
  const directoryCount = computed(() => personas.value.length)
  const pendingReviewCount = computed(
    () => suggestedIdentityCandidates.value.length + identityTraces.value.length
  )
  const selectedPersonaRelationships = computed(() => {
    const personaId = selectedPersonaId.value
    if (!personaId) return []

    return relationships.value.filter((relationship) => {
      return (
        relationship.source_entity_id === personaId ||
        relationship.target_entity_id === personaId
      )
    })
  })
  const isLoading = computed(
    () =>
      personasQuery.isLoading.value ||
      ownerPersonaQuery.isLoading.value
  )
  const isRefreshing = computed(
    () =>
      personasQuery.isFetching.value ||
      ownerPersonaQuery.isFetching.value ||
      identityCandidatesQuery.isFetching.value ||
      identityTracesQuery.isFetching.value ||
      relationshipsQuery.isFetching.value
  )
  const actionError = computed(
    () =>
      errorMessage(setOwnerPersonaMutation.error.value) ||
      errorMessage(updatePersonaAddressBookMembershipMutation.error.value) ||
      errorMessage(reviewIdentityCandidateMutation.error.value) ||
      errorMessage(assignIdentityTraceMutation.error.value)
  )
  const settingOwnerPersonaId = computed(() =>
    setOwnerPersonaMutation.isPending.value
      ? setOwnerPersonaMutation.variables.value ?? null
      : null
  )
  const reviewingCandidateId = computed(() =>
    reviewIdentityCandidateMutation.isPending.value
      ? reviewIdentityCandidateMutation.variables.value?.candidateId ?? null
      : null
  )
  const assigningTraceId = computed(() =>
    assignIdentityTraceMutation.isPending.value
      ? assignIdentityTraceMutation.variables.value?.traceId ?? null
      : null
  )

  function identityConfidence(item: PersonaIdentityCandidate | PersonaIdentity): string {
    return `${Math.round(item.confidence * 100)}%`
  }

  function languageLabel(language: string | null | undefined): string {
    if (!language) return t('Not set')
    const labels: Record<string, string> = {
      ru: t('Russian'),
      en: t('English')
    }
    return labels[language.toLowerCase()] ?? language
  }

  function trustScoreLabel(score: number | null | undefined): string {
    if (score === null || score === undefined) return t('No score')
    return `${score}/100`
  }

  function personaInitials(persona: Pick<EnrichedPersona, 'display_name' | 'email_address'>): string {
    const source = persona.display_name || persona.email_address || '?'
    return source
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((part) => part.slice(0, 1))
      .join('')
  }

  function traceTitle(trace: PersonaIdentity): string {
    return trace.identity_value || trace.identity_type
  }

  function traceKindLabel(trace: PersonaIdentity): string {
    return identityKindLabel(trace.identity_type)
  }

  function candidateTitle(candidate: PersonaIdentityCandidate): string {
    if (candidate.candidate_kind === 'attach_email_address' && candidate.email_address) {
      return candidate.email_address
    }

    if (candidate.candidate_kind === 'merge_personas') {
      return t('Possible duplicate persona')
    }

    return candidate.candidate_kind
  }

  function candidateKindLabel(candidate: PersonaIdentityCandidate): string {
    const labels: Record<string, string> = {
      attach_email_address: t('Email candidate'),
      merge_personas: t('Merge candidate'),
      split_persona: t('Split candidate')
    }
    return labels[candidate.candidate_kind] ?? candidate.candidate_kind
  }

  function formatDateTime(value: string | null | undefined): string {
    if (!value) return t('Never')
    const date = new Date(value)
    if (Number.isNaN(date.getTime())) return t('Unknown')
    return new Intl.DateTimeFormat(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    }).format(date)
  }

  function selectPersona(index: number): void {
    store.selectPersona(index)
  }

  async function refresh(): Promise<void> {
    await Promise.all([
      personasQuery.refetch(),
      ownerPersonaQuery.refetch(),
      identityCandidatesQuery.refetch(),
      identityTracesQuery.refetch(),
      relationshipsQuery.refetch()
    ])
  }

  async function setOwnerPersona(persona: EnrichedPersona): Promise<void> {
    await setOwnerPersonaMutation.mutateAsync(persona.persona_id)
  }

  async function toggleAddressBookMembership(
    persona: EnrichedPersona,
    value: boolean
  ): Promise<void> {
    await updatePersonaAddressBookMembershipMutation.mutateAsync({
      personaId: persona.persona_id,
      isAddressBook: value
    })
  }

  async function setIdentityCandidateReview(
    candidate: PersonaIdentityCandidate,
    state: PersonaIdentityReviewState
  ): Promise<void> {
    await reviewIdentityCandidateMutation.mutateAsync({
      candidateId: candidate.identity_candidate_id,
      reviewState: state
    })
  }

  async function assignTraceToOwner(trace: PersonaIdentity): Promise<void> {
    const owner = ownerPersona.value
    if (!owner) return

    await assignIdentityTraceMutation.mutateAsync({
      traceId: trace.id,
      personaId: owner.persona_id
    })
  }

  async function assignTraceToSelectedPersona(trace: PersonaIdentity): Promise<void> {
    const persona = selectedPersona.value
    if (!persona) return

    await assignIdentityTraceMutation.mutateAsync({
      traceId: trace.id,
      personaId: persona.persona_id
    })
  }

  function isSettingOwner(personaId: string): boolean {
    return (
      setOwnerPersonaMutation.isPending.value &&
      setOwnerPersonaMutation.variables.value === personaId
    )
  }

  function isReviewingCandidate(candidateId: string): boolean {
    return (
      reviewIdentityCandidateMutation.isPending.value &&
      reviewIdentityCandidateMutation.variables.value?.candidateId === candidateId
    )
  }

  function isAssigningTrace(traceId: string): boolean {
    return (
      assignIdentityTraceMutation.isPending.value &&
      assignIdentityTraceMutation.variables.value?.traceId === traceId
    )
  }

  function splitConfirmedIdentityMerge(candidate: PersonaIdentityCandidate) {
    return setIdentityCandidateReview(candidate, 'suggested')
  }

  function splitCandidateForConfirmedMerge(): PersonaIdentityCandidate | null {
    return null
  }

  return {
    activeSection,
    actionError,
    assigningTraceId,
    assignTraceToOwner,
    assignTraceToSelectedPersona,
    candidateKindLabel,
    candidateTitle,
    confirmedMergeIdentityCandidates,
    directoryCount,
    directoryFilter,
    filteredPersonas,
    formatDateTime,
    identityConfidence,
    identityTraces,
    isAssigningTrace,
    isLoading,
    isRefreshing,
    isReviewingCandidate,
    isSettingOwner,
    languageLabel,
    ownerPersona,
    pendingReviewCount,
    personaInitials,
    personaList,
    personaSearchQuery,
    personas,
    refresh,
    relationships,
    reviewingCandidateId,
    selectedPersona,
    selectedPersonaRelationships,
    selectedPersonaId,
    selectPersona,
    setIdentityCandidateReview,
    setOwnerPersona,
    settingOwnerPersonaId,
    splitCandidateForConfirmedMerge,
    splitConfirmedIdentityMerge,
    store,
    suggestedIdentityCandidates,
    toggleAddressBookMembership,
    traceKindLabel,
    traceTitle,
    trustScoreLabel
  }
}

function ownerProfile(
  owner: OwnerPersona | null | undefined,
  personas: readonly EnrichedPersona[]
): PersonaPanelProfile | null {
  if (!owner) return null

  const enriched = personas.find((persona) => persona.persona_id === owner.persona_id)
  if (enriched) {
    return { ...enriched, is_owner: true }
  }

  return {
    persona_id: owner.persona_id,
    display_name: owner.display_name,
    email_address: owner.email_address,
    language: null,
    tone: null,
    trust_score: null,
    avg_response_hours: null,
    preferred_channel: null,
    last_interaction_at: null,
    interaction_count: 0,
    frequent_topics: [],
    writing_style: null,
    persona_metadata: {},
    is_favorite: false,
    is_address_book: owner.is_address_book ?? false,
    notes: null,
    linked_projects: [],
    linked_documents: [],
    created_at: owner.created_at,
    updated_at: owner.updated_at,
    is_owner: true
  }
}

function identityKindLabel(kind: string): string {
  const labels: Record<string, string> = {
    email: 'Email',
    phone: 'Phone',
    telegram: 'Telegram',
    whatsapp: 'WhatsApp',
    social: 'Social',
    name: 'Name',
    organization: 'Organization'
  }
  return labels[kind] ?? kind
}

function errorMessage(error: unknown): string {
  if (!error) return ''
  return error instanceof Error ? error.message : String(error)
}
