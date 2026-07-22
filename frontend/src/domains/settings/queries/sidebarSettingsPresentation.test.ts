import { describe, expect, it } from 'vitest'
import { buildSidebarRuleSummaries } from './sidebarSettingsPresentation'

describe('sidebar settings presentation', () => {
  it('builds translated sidebar rules in a stable order', () => {
    expect(buildSidebarRuleSummaries((key) => `translated:${key}`)).toEqual([
      { text: 'translated:Default keeps the current sidebar order', badge: 'translated:Preset' },
      { text: 'translated:Communications sources stay nested', badge: 'translated:Context' },
      { text: 'translated:Hidden domains stay recoverable here', badge: 'translated:Safe' },
      { text: 'translated:Settings store no message content', badge: 'translated:Privacy' }
    ])
  })
})
