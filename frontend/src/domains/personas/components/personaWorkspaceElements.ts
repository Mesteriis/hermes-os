import type { TranslationFunction } from '../../../platform/i18n/types'
import type {
  EnrichedPersona,
  PersonaIdentity,
  PersonaIdentityCandidate,
  PersonaPanelProfile,
  PersonaWorkspaceSection
} from '../types/persona'

export const PERSONA_WORKSPACE_SECTIONS: readonly PersonaWorkspaceSection[] = [
  'overview',
  'communications',
  'context',
  'tasks',
  'documents',
  'notes',
  'relationships',
  'timeline',
  'dossier'
]

export function personaDirectoryRowClass(
  persona: EnrichedPersona,
  selectedPersona: PersonaPanelProfile | null,
  ownerPersona: PersonaPanelProfile | null,
  index: number
): Record<string, boolean> {
  return {
    'is-selected': selectedPersona?.persona_id === persona.persona_id,
    'is-owner': ownerPersona?.persona_id === persona.persona_id,
    'is-first': index === 0
  }
}

export function isOwnerPersona(
  persona: EnrichedPersona,
  ownerPersona: PersonaPanelProfile | null
): boolean {
  return ownerPersona?.persona_id === persona.persona_id
}

export function identityConfidence(item: { confidence: number }): string {
  return `${Math.round(item.confidence * 100)}%`
}

export function languageLabel(
  language: string | null | undefined,
  t: TranslationFunction
): string {
  if (!language) return t('Not set')
  const labels: Record<string, string> = {
    ru: t('Russian'),
    en: t('English')
  }
  return labels[language.toLowerCase()] ?? language
}

export function trustScoreLabel(
  score: number | null | undefined,
  t: TranslationFunction
): string {
  if (score === null || score === undefined) return t('No score')
  return `${score}/100`
}

export function healthScoreLabel(
  score: number | null | undefined,
  t: TranslationFunction
): string {
  if (score === null || score === undefined) return t('No score')
  const healthScore = Math.min(100, score + 8)
  return `${healthScore}/100`
}

export function sectionLabel(
  section: PersonaWorkspaceSection,
  t: TranslationFunction
): string {
  const labels: Record<PersonaWorkspaceSection, string> = {
    overview: t('Overview'),
    communications: t('Communications'),
    context: t('Context'),
    tasks: t('Tasks'),
    documents: t('Documents'),
    notes: t('Notes'),
    relationships: t('Relationships'),
    timeline: t('Timeline'),
    dossier: t('Dossier')
  }
  return labels[section]
}

export function sectionUnavailableMessage(
  section: PersonaWorkspaceSection,
  t: TranslationFunction
): string {
  const labels: Record<PersonaWorkspaceSection, string> = {
    overview: t('Overview is available from the current persona projection.'),
    communications: t('Persona communication projection is not wired into this workspace yet.'),
    context: t('Facts, memory cards and preferences endpoints exist, but this tab has no frontend query yet.'),
    tasks: t('Task candidates must arrive through Review promotion before this tab can show durable tasks.'),
    documents: t('Persona document links are present as ids only; document cards are not connected here yet.'),
    notes: t('Notes are available from the current persona record.'),
    relationships: t('Relationships are available from the relationships projection.'),
    timeline: t('Persona timeline endpoint exists, but this tab is not wired into the surface yet.'),
    dossier: t('Dossier endpoint exists, but reviewed dossier data is not loaded in this story surface yet.')
  }
  return labels[section]
}

export function personaInitials(
  persona: Pick<EnrichedPersona, 'display_name' | 'email_address'>
): string {
  const source = persona.display_name || persona.email_address || '?'
  const parts = source.split(/\s+/)
  let initials = ''

  for (const part of parts) {
    if (!part) continue
    initials += part.slice(0, 1)
    if (initials.length >= 2) break
  }

  return initials
}

export function traceTitle(trace: PersonaIdentity): string {
  return trace.identity_value || trace.identity_type
}

export function traceKindLabel(trace: PersonaIdentity): string {
  const labels: Record<string, string> = {
    email: 'Email',
    phone: 'Phone',
    telegram: 'Telegram',
    whatsapp: 'WhatsApp',
    social: 'Social',
    name: 'Name',
    organization: 'Organization'
  }
  return labels[trace.identity_type] ?? trace.identity_type
}

export function candidateTitle(
  candidate: PersonaIdentityCandidate,
  t: TranslationFunction
): string {
  if (candidate.candidate_kind === 'attach_email_address' && candidate.email_address) {
    return candidate.email_address
  }

  if (candidate.candidate_kind === 'merge_personas') {
    return t('Possible duplicate persona')
  }

  return candidate.candidate_kind
}

export function candidateKindLabel(
  candidate: PersonaIdentityCandidate,
  t: TranslationFunction
): string {
  const labels: Record<string, string> = {
    attach_email_address: t('Email candidate'),
    merge_personas: t('Merge candidate'),
    split_persona: t('Split candidate')
  }
  return labels[candidate.candidate_kind] ?? candidate.candidate_kind
}

export function formatDateTime(
  value: string | null | undefined,
  t: TranslationFunction
): string {
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
