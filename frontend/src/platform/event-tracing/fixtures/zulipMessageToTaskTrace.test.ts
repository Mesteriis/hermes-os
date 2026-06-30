import { describe, expect, it } from 'vitest'
import {
  zulipMessageToTaskTraceEventTypes,
  zulipMessageToTaskTraceFixture,
} from './zulipMessageToTaskTrace'

describe('Zulip message-to-task trace fixture', () => {
  it('models the provider-neutral UI trace from Zulip raw signal to Review candidate', () => {
    expect(zulipMessageToTaskTraceFixture.events.map((item) => item.event.event_type)).toEqual([
      ...zulipMessageToTaskTraceEventTypes,
    ])
    expect(zulipMessageToTaskTraceFixture.edges).toEqual([
      {
        parent_event_id: 'event:fixture:zulip:raw-message',
        child_event_id: 'event:fixture:zulip:accepted-message',
      },
      {
        parent_event_id: 'event:fixture:zulip:accepted-message',
        child_event_id: 'event:fixture:communication:message-recorded',
      },
      {
        parent_event_id: 'event:fixture:communication:message-recorded',
        child_event_id: 'event:fixture:review:task-candidate',
      },
      {
        parent_event_id: 'event:fixture:review:task-candidate',
        child_event_id: 'event:fixture:review:item-available',
      },
    ])
    expect(zulipMessageToTaskTraceFixture.dead_letters).toEqual([])
    expect(zulipMessageToTaskTraceFixture.missing_parent_ids).toEqual([])
  })

  it('keeps the UI fixture sanitized and credential-free', () => {
    const serialized = JSON.stringify(zulipMessageToTaskTraceFixture).toLowerCase()

    expect(serialized).not.toContain('api_key')
    expect(serialized).not.toContain('authorization')
    expect(serialized).not.toContain('basic ')
    expect(serialized).not.toContain('password')
    expect(serialized).not.toContain('secret')
    expect(serialized).not.toContain('token')
    expect(serialized).toContain('"credential_payload_present":false')
  })
})
