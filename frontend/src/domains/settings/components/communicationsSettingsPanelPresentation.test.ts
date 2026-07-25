import { describe, expect, it } from 'vitest'
import {
  communicationMappingSourceLabel,
  formatCommunicationTimestamp,
  nullableLocalFolder,
  parseSemanticRole,
  semanticRoles
} from './communicationsSettingsPanelPresentation'

describe('communications settings panel presentation', () => {
  it('validates provider semantic roles and nullable folders', () => {
    expect(semanticRoles).toHaveLength(10)
    expect(parseSemanticRole('inbox')).toBe('inbox')
    expect(parseSemanticRole('unknown')).toBeNull()
    expect(nullableLocalFolder('')).toBeNull()
    expect(nullableLocalFolder('Archive')).toBe('Archive')
  })

  it('formats command diagnostics and mapping labels', () => {
    expect(communicationMappingSourceLabel('manual', (key) => `translated:${key}`))
      .toBe('translated:Manual override')
    expect(formatCommunicationTimestamp(null)).toBe('—')
  })
})
