import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  assignIdentityTrace,
  fetchIdentityCandidates,
  fetchIdentityTraces,
  fetchOwnerPersona,
  fetchPersonas,
  fetchRelationships,
  normalizePersonaReadModel,
  reviewIdentityCandidate,
  setOwnerPersona,
  updatePersonaAddressBookMembership
} from '../api/personas'
import type {
  EnrichedPersona,
  OwnerPersona,
  PersonaIdentityCandidate,
  PersonaIdentity,
  PersonaIdentityReviewState,
  Relationship
} from '../types/persona'

export const personasQueryKeys = {
  all: ['personas'] as const,
  list: ['personas', 'list'] as const,
  owner: ['personas', 'owner'] as const,
  identityCandidates: ['personas', 'identity-candidates'] as const,
  identityTraces: ['personas', 'identity-traces'] as const,
  relationships: ['personas', 'relationships'] as const
}

export function usePersonasQuery() {
  return useQuery<EnrichedPersona[]>({
    queryKey: personasQueryKeys.list,
    queryFn: async () => {
      const res = await fetchPersonas(50)
      return res.items.map((persona) => ({
        ...normalizePersonaReadModel(persona),
        is_address_book: persona.is_address_book ?? false
      }))
    }
  })
}

export function useOwnerPersonaQuery() {
  return useQuery<OwnerPersona | null>({
    queryKey: personasQueryKeys.owner,
    queryFn: async () => {
      const res = await fetchOwnerPersona()
      return res.owner_persona
    }
  })
}

export function useIdentityCandidatesQuery() {
  return useQuery<PersonaIdentityCandidate[]>({
    queryKey: personasQueryKeys.identityCandidates,
    queryFn: async () => {
      const res = await fetchIdentityCandidates(50)
      return res.items
    }
  })
}

export function useIdentityTracesQuery() {
  return useQuery<PersonaIdentity[]>({
    queryKey: personasQueryKeys.identityTraces,
    queryFn: async () => {
      const res = await fetchIdentityTraces(50)
      return res.items
    }
  })
}

export function useRelationshipsQuery(personaId: MaybeRefOrGetter<string | null>) {
  return useQuery<Relationship[]>({
    queryKey: computed(() => [...personasQueryKeys.relationships, toValue(personaId)] as const),
    queryFn: async () => {
      const currentPersonaId = toValue(personaId)
      if (!currentPersonaId) return []

      const res = await fetchRelationships({
        entityKind: 'persona',
        entityId: currentPersonaId,
        limit: 50
      })
      return res.items
    },
    enabled: computed(() => Boolean(toValue(personaId)))
  })
}

export function useSetOwnerPersonaMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (personaId: string) => setOwnerPersona(personaId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: personasQueryKeys.all })
    }
  })
}

export function useUpdatePersonaAddressBookMembershipMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ personaId, isAddressBook }: { personaId: string; isAddressBook: boolean }) =>
      updatePersonaAddressBookMembership(personaId, isAddressBook),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: personasQueryKeys.all })
    }
  })
}

export function useReviewIdentityCandidateMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      candidateId,
      reviewState
    }: {
      candidateId: string
      reviewState: PersonaIdentityReviewState
    }) => reviewIdentityCandidate(candidateId, reviewState),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: personasQueryKeys.all })
    }
  })
}

export function useAssignIdentityTraceMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ traceId, personaId }: { traceId: string; personaId: string }) =>
      assignIdentityTrace(traceId, personaId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: personasQueryKeys.all })
    }
  })
}
