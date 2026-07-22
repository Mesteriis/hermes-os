import type { PersonaIdentity } from '../types/persona'

export function canAssignIdentityTraceToPersona(
  trace: Pick<PersonaIdentity, 'persona_id'>,
  personaId: string
): boolean {
  return trace.persona_id !== personaId
}

export function canAssignIdentityTraceToOwner(
  trace: Pick<PersonaIdentity, 'persona_id'>,
  ownerPersonaId: string | null
): boolean {
  return ownerPersonaId !== null && trace.persona_id !== ownerPersonaId
}

export function isReviewingIdentityCandidate(
  candidateId: string,
  reviewingCandidateId: string | null
): boolean {
  return reviewingCandidateId === candidateId
}

export function isAssigningIdentityTrace(
  traceId: string,
  assigningTraceId: string | null
): boolean {
  return assigningTraceId === traceId
}
