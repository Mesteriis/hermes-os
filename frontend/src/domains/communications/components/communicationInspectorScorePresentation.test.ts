import { describe, expect, it } from 'vitest'
import {
  communicationInspectorScoreTone,
  communicationInspectorScoreUnit,
} from './communicationInspectorScorePresentation'

describe('communication inspector score presentation', () => {
  it('shares score thresholds and protects zero max scores', () => {
    expect(communicationInspectorScoreTone(8, 10)).toBe('success')
    expect(communicationInspectorScoreTone(6, 10)).toBe('warning')
    expect(communicationInspectorScoreTone(5, 10)).toBe('danger')
    expect(communicationInspectorScoreTone(1, 0)).toBe('danger')
    expect(communicationInspectorScoreUnit(10)).toBe('/10')
  })
})
