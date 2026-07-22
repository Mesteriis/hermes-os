import { describe, expect, it } from 'vitest'
import { buildPersonaEntityLabels } from './personaWorkspacePresentation'
import type { EnrichedPersona, PersonaPanelProfile } from '../types/persona'

describe('persona workspace presentation', () => {
  it('builds entity labels with selected context taking precedence', () => {
    expect(buildPersonaEntityLabels(
      [{ persona_id: 'persona-1', display_name: 'Directory name' } satisfies Pick<EnrichedPersona, 'persona_id' | 'display_name'>],
      { persona_id: 'persona-1', display_name: 'Owner name' } satisfies Pick<PersonaPanelProfile, 'persona_id' | 'display_name'>,
      { persona_id: 'persona-1', display_name: 'Selected name' } satisfies Pick<PersonaPanelProfile, 'persona_id' | 'display_name'>,
    )).toEqual({ 'persona-1': 'Selected name' })
  })
})
