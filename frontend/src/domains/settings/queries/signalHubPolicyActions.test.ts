import { describe, expect, it, vi } from 'vitest'
import {
  buildSignalHubSourcePolicyRequest,
  clearSignalHubPolicy,
  createSignalHubPolicy,
  type SignalHubPolicyDraft
} from './signalHubPolicyActions'

describe('signal hub policy actions', () => {
  it('routes disabled source policies to source control', async () => {
    const dependencies = dependenciesFor()

    await createSignalHubPolicy(draft('source'), 'disabled', dependencies)

    expect(dependencies.disableSource).toHaveBeenCalledWith('telegram')
    expect(dependencies.disable).not.toHaveBeenCalled()
    expect(dependencies.create).not.toHaveBeenCalled()
  })

  it('clears muted policy through unmute with the policy context', async () => {
    const dependencies = dependenciesFor()

    await clearSignalHubPolicy({ ...draft('connection'), mode: 'muted' }, dependencies)

    expect(dependencies.unmute).toHaveBeenCalledWith(draft('connection'))
    expect(dependencies.enable).not.toHaveBeenCalled()
  })

  it('builds a normalized source policy request', () => {
    expect(buildSignalHubSourcePolicyRequest('telegram', 'pause')).toEqual({
      scope: 'source', source_code: 'telegram', connection_id: null,
      event_pattern: null, reason: 'pause'
    })
  })
})

function draft(scope: SignalHubPolicyDraft['scope']): SignalHubPolicyDraft {
  return {
    scope,
    source_code: 'telegram',
    connection_id: scope === 'connection' ? 'connection-1' : null,
    event_pattern: scope === 'event_pattern' ? 'signal.raw.*' : null,
    reason: 'settings test'
  }
}

function dependenciesFor() {
  return {
    pause: vi.fn().mockResolvedValue(undefined),
    mute: vi.fn().mockResolvedValue(undefined),
    disableSource: vi.fn().mockResolvedValue(undefined),
    disable: vi.fn().mockResolvedValue(undefined),
    create: vi.fn().mockResolvedValue(undefined),
    resume: vi.fn().mockResolvedValue(undefined),
    unmute: vi.fn().mockResolvedValue(undefined),
    enableSource: vi.fn().mockResolvedValue(undefined),
    enable: vi.fn().mockResolvedValue(undefined)
  }
}
