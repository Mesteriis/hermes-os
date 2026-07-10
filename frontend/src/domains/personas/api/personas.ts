import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  EnrichedPersona,
  OwnerPersona,
  PersonaDossier,
  PersonaIdentityCandidate,
  PersonaIdentity,
  PersonaIdentityReviewState,
  PersonaReadModel,
  Relationship
} from '../types/persona'

export type PersonaListResponse = { items: PersonaReadModel[] }
export type OwnerPersonaResponse = { owner_persona: OwnerPersona | null }
export type PersonaIdentityCandidateListResponse = { items: PersonaIdentityCandidate[] }
export type PersonaIdentityTraceListResponse = { items: PersonaIdentity[] }
export type RelationshipListResponse = { items: Relationship[] }

export async function fetchPersonas(limit = 50): Promise<PersonaListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<PersonaListResponse>(
    `/api/v1/personas?${params.toString()}`,
    'Personas request failed'
  )
}

export async function fetchOwnerPersona(): Promise<OwnerPersonaResponse> {
  const response = await ApiClient.instance.get<{ owner_persona: PersonaReadModel | OwnerPersona | null }>(
    '/api/v1/personas/owner',
    'Owner persona request failed'
  )

  return {
    owner_persona: response.owner_persona ? normalizeOwnerPersona(response.owner_persona) : null
  }
}

export async function setOwnerPersona(personaId: string): Promise<OwnerPersonaResponse> {
  const response = await ApiClient.instance.put<{ owner_persona: PersonaReadModel | OwnerPersona | null }>(
    '/api/v1/personas/owner',
    { persona_id: personaId },
    'Owner persona update failed'
  )

  return {
    owner_persona: response.owner_persona ? normalizeOwnerPersona(response.owner_persona) : null
  }
}

export async function updatePersonaAddressBookMembership(
  personaId: string,
  isAddressBook: boolean
): Promise<OwnerPersona> {
  const response = await ApiClient.instance.put<PersonaReadModel | OwnerPersona>(
    `/api/v1/personas/${encodeURIComponent(personaId)}/address-book`,
    { is_address_book: isAddressBook },
    'Persona address book update failed'
  )
  return normalizeOwnerPersona(response)
}

export async function fetchPersonaDossier(personaId: string): Promise<PersonaDossier> {
  return ApiClient.instance.get<PersonaDossier>(
    `/api/v1/personas/${encodeURIComponent(personaId)}/dossier`,
    'Persona dossier request failed'
  )
}

export async function fetchIdentityCandidates(limit = 50): Promise<PersonaIdentityCandidateListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<PersonaIdentityCandidateListResponse>(
    `/api/v1/identity-candidates?${params.toString()}`,
    'Identity candidate request failed'
  )
}

export async function reviewIdentityCandidate(
  identityCandidateId: string,
  reviewState: PersonaIdentityReviewState
): Promise<void> {
  await ApiClient.instance.put(
    `/api/v1/identity-candidates/${encodeURIComponent(identityCandidateId)}/review`,
    { review_state: reviewState }
  )
}

export async function fetchIdentityTraces(limit = 50): Promise<PersonaIdentityTraceListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<PersonaIdentityTraceListResponse>(
    `/api/v1/identity-traces?${params.toString()}`,
    'Identity traces request failed'
  )
}

export async function assignIdentityTrace(traceId: string, personaId: string): Promise<void> {
  await ApiClient.instance.put(
    `/api/v1/identity-traces/${encodeURIComponent(traceId)}/assignment`,
    { persona_id: personaId },
    'Identity trace assignment failed'
  )
}

export function normalizePersonaReadModel(persona: PersonaReadModel | OwnerPersona): EnrichedPersona {
  const personaId = persona.persona_id
  const displayName =
    'identity' in persona
      ? persona.identity.display_name
      : persona.display_name
  const emailAddress =
    'identity' in persona
      ? persona.identity.email_address
      : persona.email_address

  return {
    persona_id: personaId,
    display_name: displayName,
    email_address: emailAddress,
    language: null,
    tone: null,
    trust_score: null,
    avg_response_hours: null,
    preferred_channel: emailAddress ? 'mail' : null,
    last_interaction_at: null,
    interaction_count: 0,
    frequent_topics: [],
    writing_style: null,
    persona_metadata: {},
    is_favorite: false,
    is_address_book: persona.is_address_book ?? false,
    notes: null,
    linked_projects: [],
    linked_documents: [],
    created_at: persona.created_at,
    updated_at: persona.updated_at
  }
}

export function normalizeOwnerPersona(persona: PersonaReadModel | OwnerPersona): OwnerPersona {
  const personaId = persona.persona_id
  const displayName =
    'identity' in persona
      ? persona.identity.display_name
      : persona.display_name
  const emailAddress =
    'identity' in persona
      ? persona.identity.email_address
      : persona.email_address

  return {
    persona_id: personaId,
    display_name: displayName,
    email_address: emailAddress,
    persona_type: persona.persona_type,
    is_self: persona.is_self,
    is_address_book: persona.is_address_book ?? false,
    created_at: persona.created_at,
    updated_at: persona.updated_at
  }
}

export async function fetchRelationships(params: {
  entityKind: 'persona'
  entityId: string
  limit?: number
}): Promise<RelationshipListResponse> {
  const query = new URLSearchParams({
    entity_kind: params.entityKind,
    entity_id: params.entityId,
    limit: String(Math.trunc(params.limit ?? 50))
  })
  return ApiClient.instance.get<RelationshipListResponse>(
    `/api/v1/relationships?${query.toString()}`,
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
