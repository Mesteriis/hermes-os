import type { EntityIconKind, UtilityTone } from '@/shared/ui'

export type MailInspectorCheck = {
  id: string
  label: string
  description: string
  tone?: UtilityTone
  icon?: string
}

export type MailInspectorIntelligence = {
  score: number
  maxScore: number
  label: string
  summary: string
  checks: readonly MailInspectorCheck[]
}

export type MailInspectorEntityItem = {
  id: string
  entity: EntityIconKind
  title: string
  description: string
  evidenceLabel?: string
  tone?: UtilityTone
}

export type MailInspectorEntityGroup = {
  id: string
  title: string
  items: readonly MailInspectorEntityItem[]
}

export type MailInspectorTopic = {
  id: string
  label: string
  tone?: UtilityTone
}

export type MailInspectorSemanticFact = {
  id: string
  label: string
  value: string
  tone?: UtilityTone
}

export type MailInspectorActionItem = {
  id: string
  label: string
  description: string
  icon: string
  tone?: UtilityTone
  contract?: string
}

export type MailInspectorContextItem = {
  id: string
  title: string
  description: string
  icon: string
  tone?: UtilityTone
}

export type MailInspectorModel = {
  intelligence: MailInspectorIntelligence
  entityGroups: readonly MailInspectorEntityGroup[]
  topics: readonly MailInspectorTopic[]
  semanticFacts: readonly MailInspectorSemanticFact[]
  suggestedActions: readonly MailInspectorActionItem[]
  relatedContext: readonly MailInspectorContextItem[]
}
