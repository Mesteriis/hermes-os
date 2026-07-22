import { describe, expect, it } from 'vitest'
import { signalHubViewPresentation } from './signalHubSettingsPresentation'

describe('Signal Hub view presentation', () => {
  it('provides graph metadata', () => {
    expect(signalHubViewPresentation('graph')).toEqual({
      icon: 'tabler:route',
      ariaLabel: 'Signal consumer graph',
      eyebrow: 'Graph',
      title: 'Signals and consumers',
      description: 'Raw and accepted signal routes from the current Signal Hub surface.',
    })
  })

  it('provides inventory metadata', () => {
    expect(signalHubViewPresentation('inventory')).toEqual({
      icon: 'tabler:table',
      ariaLabel: 'Signal inventory',
      eyebrow: 'Inventory',
      title: 'All signals',
      description: 'Source-scoped pause, mute, disable and resume controls.',
    })
  })
})
