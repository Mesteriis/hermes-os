import { ApiClient } from '../../../platform/api/ApiClient'
import type { EnrichedPerson, PersonDossier, PersonIdentityCandidate, PersonIdentity, PersonIdentityReviewState, Relationship } from '../types/persona'

export type PersonListResponse = { items: EnrichedPerson[] }
export type PersonIdentityCandidateListResponse = { items: PersonIdentityCandidate[] }
export type PersonIdentityTraceListResponse = { items: PersonIdentity[] }
export type OrganizationListResponse = { items: any[] }
export type RelationshipListResponse = { relationships: Relationship[] }

export async function fetchPersons(limit = 50): Promise<PersonListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<PersonListResponse>(
    `/api/v1/persons?${params.toString()}`,
    'Persons request failed'
  )
}

export async function fetchPersonDossier(personId: string): Promise<PersonDossier> {
  return ApiClient.instance.get<PersonDossier>(
    `/api/v1/persons/${encodeURIComponent(personId)}/dossier`,
    'Person dossier request failed'
  )
}

export async function fetchIdentityCandidates(limit = 50): Promise<PersonIdentityCandidateListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<PersonIdentityCandidateListResponse>(
    `/api/v1/identity-candidates?${params.toString()}`,
    'Identity candidate request failed'
  )
}

export async function reviewIdentityCandidate(
  identityCandidateId: string,
  reviewState: PersonIdentityReviewState
): Promise<void> {
  await ApiClient.instance.put(
    `/api/v1/identity-candidates/${encodeURIComponent(identityCandidateId)}/review`,
    { review_state: reviewState }
  )
}

export async function fetchIdentityTraces(limit = 50): Promise<PersonIdentityTraceListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<PersonIdentityTraceListResponse>(
    `/api/v1/identity-traces?${params.toString()}`,
    'Identity traces request failed'
  )
}

export async function assignIdentityTrace(traceId: string, personId: string): Promise<void> {
  await ApiClient.instance.post(
    `/api/v1/identity-traces/${encodeURIComponent(traceId)}/assign`,
    { person_id: personId }
  )
}

export async function fetchRelationships(limit = 50): Promise<RelationshipListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<RelationshipListResponse>(
    `/api/v1/relationships?${params.toString()}`,
    'Relationships request failed'
  )
}

export async function reviewRelationship(
  relationshipId: string,
  reviewState: string
): Promise<void> {
  await ApiClient.instance.put(
    `/api/v1/relationships/${encodeURIComponent(relationshipId)}/review`,
    { review_state: reviewState }
  )
}

export async function fetchOrganizations(limit = 50): Promise<OrganizationListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<OrganizationListResponse>(
    `/api/v1/organizations?${params.toString()}`,
    'Organizations request failed'
  )
}

export async function fetchOrganization(orgId: string): Promise<any> {
  return ApiClient.instance.get<any>(
    `/api/v1/organizations/${encodeURIComponent(orgId)}`,
    'Organization request failed'
  )
}
