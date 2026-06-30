import { describe, expect, it } from 'vitest'
import { signalHubKeys } from './useSignalHubQuery'

describe('Signal Hub query keys', () => {
  it('keeps all Signal Hub queries under one stable namespace', () => {
    expect(signalHubKeys.all).toEqual(['signal-hub'])
    expect(signalHubKeys.sources()).toEqual(['signal-hub', 'sources'])
    expect(signalHubKeys.connections()).toEqual(['signal-hub', 'connections'])
    expect(signalHubKeys.runtimes()).toEqual(['signal-hub', 'runtimes'])
    expect(signalHubKeys.health()).toEqual(['signal-hub', 'health'])
    expect(signalHubKeys.replay()).toEqual(['signal-hub', 'replay'])
    expect(signalHubKeys.policies()).toEqual(['signal-hub', 'policies'])
    expect(signalHubKeys.profiles()).toEqual(['signal-hub', 'profiles'])
  })
})
