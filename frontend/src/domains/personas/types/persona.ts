export type PersonaType = 'human' | 'ai_agent' | 'organization_proxy' | 'system'

export interface EnrichedPerson {
  person_id: string
  display_name: string
  email_address: string
  preferred_channel: string | null
  last_interaction_at: string | null
  linked_projects: string[] | null
}

export interface PersonDossier {
  summary: string
  source_refs: string[]
  generated_at: string
}

export interface PersonIdentityCandidate {
  candidate_id: string
  candidate_kind: string
  left_person_id: string
  right_person_id: string | null
  evidence_summary: string
  confidence: number
  review_state: string
  created_at: string
}

export type PersonIdentityReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'

export interface PersonIdentity {
  id: string
  identity_type: string
  value: string
  source: string
  confidence: number
  person_id: string | null
}

export interface Relationship {
  relationship_id: string
  source_entity_id: string
  source_entity_kind: string
  target_entity_id: string
  target_entity_kind: string
  relationship_type: string
  trust_score: number
  strength_score: number
  confidence: number
  review_state: string
}

export type RelationshipReviewState = 'suggested' | 'system_accepted' | 'user_confirmed' | 'user_rejected'

export interface PersonItem {
  person_id: string
  name: string
  role: string
  company: string
  channel?: string
  status?: string
}

export interface PersonaOption {
  person_id: string
  name: string
  company: string
}
