import { describe, expect, it } from 'vitest'
import {
  buildSignalHubReplayRequest,
  describeSignalHubReplayRequest
} from './signalHubReplay'

describe('signalHubReplay helpers', () => {
  it('builds a replay request without selectors for full pattern replay', () => {
    const request = buildSignalHubReplayRequest({
      source_code: ' telegram ',
      event_pattern: ' signal.raw.telegram.* ',
      target_consumer: ' signal_hub_raw_signal_dispatcher ',
      selector_mode: 'all'
    })

    expect(request).toMatchObject({
      source_code: 'telegram',
      event_pattern: 'signal.raw.telegram.*',
      target_consumer: 'signal_hub_raw_signal_dispatcher',
      metadata: {
        requested_from: 'settings_signal_hub',
        selector_mode: 'all'
      }
    })
    expect('from_position' in request).toBe(false)
    expect('to_position' in request).toBe(false)
    expect('from_time' in request).toBe(false)
    expect('to_time' in request).toBe(false)
  })

  it('builds a replay request with position selectors', () => {
    const request = buildSignalHubReplayRequest({
      source_code: 'telegram',
      event_pattern: 'signal.raw.telegram.*',
      selector_mode: 'position',
      from_position: '10',
      to_position: '20'
    })

    expect(request.from_position).toBe(10n)
    expect(request.to_position).toBe(20n)
    expect(request.from_time).toBeUndefined()
    expect(request.to_time).toBeUndefined()
  })

  it('builds a replay request with time selectors', () => {
    const request = buildSignalHubReplayRequest({
      source_code: 'telegram',
      event_pattern: 'signal.raw.telegram.*',
      selector_mode: 'time',
      from_time: '2026-06-23T00:00:00Z',
      to_time: '2026-06-23T01:00:00Z'
    })

    expect(request.from_position).toBeUndefined()
    expect(request.to_position).toBeUndefined()
    expect(request.from_time).toBe('2026-06-23T00:00:00Z')
    expect(request.to_time).toBe('2026-06-23T01:00:00Z')
  })

  it('describes replay selectors for the list row', () => {
    const description = describeSignalHubReplayRequest({
      id: 'replay-1',
      source_code: 'telegram',
      connection_id: null,
      event_pattern: 'signal.raw.telegram.*',
      from_position: 10n,
      to_position: 20n,
      from_time: null,
      to_time: null,
      target_consumer: 'signal_hub_raw_signal_dispatcher',
      target_projection: 'timeline_event_log',
      status: 'queued',
      requested_by: 'hermes-frontend',
      requested_at: '2026-06-23T00:00:00Z',
      started_at: null,
      completed_at: null,
      last_error_redacted: null,
      replayed_count: 0,
      metadata: {}
    })

    expect(description).toBe(
      'pos 10..20 / consumer signal_hub_raw_signal_dispatcher / projection timeline_event_log / 2026-06-23T00:00:00Z'
    )
  })

  it('builds a projection-targeted replay request with optional source filters', () => {
    const request = buildSignalHubReplayRequest({
      source_code: '',
      event_pattern: '',
      target_projection: ' communication_messages ',
      selector_mode: 'all'
    })

    expect(request.source_code).toBeNull()
    expect(request.event_pattern).toBeNull()
    expect(request.target_projection).toBe('communication_messages')
  })

  it('builds a connection-scoped replay request', () => {
    const request = buildSignalHubReplayRequest({
      source_code: ' mail ',
      connection_id: ' conn-1 ',
      event_pattern: ' signal.raw.mail.* ',
      selector_mode: 'all'
    })

    expect(request.source_code).toBe('mail')
    expect(request.connection_id).toBe('conn-1')
    expect(request.event_pattern).toBe('signal.raw.mail.*')
  })

  it('includes connection scope in replay description', () => {
    const description = describeSignalHubReplayRequest({
      id: 'replay-2',
      source_code: 'mail',
      connection_id: 'conn-1',
      event_pattern: 'signal.raw.mail.*',
      from_position: null,
      to_position: null,
      from_time: null,
      to_time: null,
      target_consumer: null,
      target_projection: null,
      status: 'queued',
      requested_by: 'hermes-frontend',
      requested_at: '2026-06-23T00:00:00Z',
      started_at: null,
      completed_at: null,
      last_error_redacted: null,
      replayed_count: 0,
      metadata: {}
    })

    expect(description).toBe('connection conn-1 / 2026-06-23T00:00:00Z')
  })
})
