import { useQuery } from '@tanstack/vue-query'
import { fetchPersons, fetchIdentityCandidates, fetchIdentityTraces, fetchRelationships } from '../api/personas'
import type { EnrichedPerson, PersonIdentityCandidate, PersonIdentity, Relationship } from '../types/persona'

export function usePersonsQuery() {
  return useQuery<EnrichedPerson[]>({
    queryKey: ['persons', 'list'],
    queryFn: async () => {
      const res = await fetchPersons(50)
      return res.items
    }
  })
}

export function useIdentityCandidatesQuery() {
  return useQuery<PersonIdentityCandidate[]>({
    queryKey: ['persons', 'identity-candidates'],
    queryFn: async () => {
      const res = await fetchIdentityCandidates(50)
      return res.items
    }
  })
}

export function useIdentityTracesQuery() {
  return useQuery<PersonIdentity[]>({
    queryKey: ['persons', 'identity-traces'],
    queryFn: async () => {
      const res = await fetchIdentityTraces(50)
      return res.items
    }
  })
}

export function useRelationshipsQuery() {
  return useQuery<Relationship[]>({
    queryKey: ['persons', 'relationships'],
    queryFn: async () => {
      const res = await fetchRelationships(50)
      return res.relationships
    }
  })
}
