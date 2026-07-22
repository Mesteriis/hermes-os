import { describe, expect, it } from 'vitest'
import { buildAiSettingsTabs } from './aiSettingsPanelPresentation'

describe('AI settings tabs', () => {
  it('builds translated tabs with owner counts', () => {
    expect(buildAiSettingsTabs(
      { providers: 1, models: 2, routes: 3, stats: 4 },
      (key) => `translated:${key}`,
    )).toEqual([
      { id: 'providers', icon: 'tabler:plug-connected', label: 'translated:Provider setup', count: 1 },
      { id: 'models', icon: 'tabler:list-search', label: 'translated:Model catalog', count: 2 },
      { id: 'routes', icon: 'tabler:route', label: 'translated:Action routing', count: 3 },
      { id: 'stats', icon: 'tabler:chart-histogram', label: 'translated:Usage statistics', count: 4 },
    ])
  })
})
