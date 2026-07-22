import { describe, expect, it } from 'vitest'
import {
  parseForwardingSeverity,
  parseSemanticRole,
} from './communicationsSettingsPanelPresentation'

describe('communications settings semantic parsing', () => {
  it('parses known semantic roles without assertions', () => {
    expect(parseSemanticRole('inbox')).toBe('inbox')
    expect(parseSemanticRole('unknown')).toBeNull()
  })

  it('parses forwarding severity and defaults invalid input safely', () => {
    expect(parseForwardingSeverity('critical')).toBe('critical')
    expect(parseForwardingSeverity('invalid')).toBe('high')
  })
})
