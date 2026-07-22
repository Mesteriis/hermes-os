import { describe, expect, it } from 'vitest'
import { relationshipGraphEdgeData } from './personaRelationshipGraphModel'

describe('persona relationship graph model', () => {
  it('accepts complete edge data', () => {
    expect(relationshipGraphEdgeData({
      relationshipId: 'relationship-1',
      type: 'colleague',
      state: 'confirmed',
      confidence: 0.9,
      sourceTitle: 'Alice',
      targetTitle: 'Acme',
      icon: 'tabler:briefcase',
      iconLabel: 'Colleague',
    })).toEqual({
      relationshipId: 'relationship-1',
      type: 'colleague',
      state: 'confirmed',
      confidence: 0.9,
      sourceTitle: 'Alice',
      targetTitle: 'Acme',
      icon: 'tabler:briefcase',
      iconLabel: 'Colleague',
    })
  })

  it('rejects malformed edge data', () => {
    expect(relationshipGraphEdgeData({ relationshipId: 'relationship-1' })).toBeNull()
    expect(relationshipGraphEdgeData(null)).toBeNull()
  })
})
