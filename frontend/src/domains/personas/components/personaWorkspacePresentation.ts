import type { EnrichedPersona, PersonaPanelProfile } from '../types/persona'

export function buildPersonaEntityLabels(
  personas: readonly Pick<EnrichedPersona, 'persona_id' | 'display_name'>[],
  ownerPersona: Pick<PersonaPanelProfile, 'persona_id' | 'display_name'> | null,
  selectedPersona: Pick<PersonaPanelProfile, 'persona_id' | 'display_name'> | null
): Record<string, string> {
  const labels = new Map<string, string>()
  for (const persona of personas) labels.set(persona.persona_id, persona.display_name)
  if (ownerPersona) labels.set(ownerPersona.persona_id, ownerPersona.display_name)
  if (selectedPersona) labels.set(selectedPersona.persona_id, selectedPersona.display_name)
  return Object.fromEntries(labels)
}
