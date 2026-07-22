import { describe, expect, it } from 'vitest'
import {
  buildSignalHubProfilePolicy,
  buildSignalHubProfileSaveRequest
} from './signalHubProfileRequests'

describe('signal hub profile requests', () => {
  it('normalizes profile policy fields according to scope', () => {
    expect(buildSignalHubProfilePolicy({
      scope: 'connection',
      sourceCode: 'telegram',
      connectionId: '',
      eventPattern: 'ignored',
      mode: 'muted',
      reason: 'owner rule'
    })).toEqual({
      scope: 'connection',
      source_code: 'telegram',
      connection_id: null,
      event_pattern: null,
      mode: 'muted',
      reason: 'owner rule'
    })
  })

  it('builds the shared profile save request', () => {
    const policies = [buildSignalHubProfilePolicy({
      scope: 'source', sourceCode: 'telegram', connectionId: '',
      eventPattern: '', mode: 'paused', reason: 'maintenance'
    })]
    expect(buildSignalHubProfileSaveRequest('Primary', 'Description', policies)).toEqual({
      display_name: 'Primary', description: 'Description', source_policies: policies
    })
  })
})
