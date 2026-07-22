import { describe, expect, it } from 'vitest'
import { mapCalendarSearchResponse } from './calendarResponseMapping'

const event = {
  event_id: 'event-1',
  source_event_id: null,
  account_id: 'account-1',
  source_id: 'source-1',
  title: 'Planning',
  description: null,
  location: null,
  start_at: '2026-07-21T09:00:00Z',
  end_at: '2026-07-21T10:00:00Z',
  timezone: 'UTC',
  all_day: false,
  recurrence_rule: null,
  status: 'confirmed',
  visibility: 'private',
  event_type: 'meeting',
  importance_score: null,
  readiness_score: null,
  sync_status: 'synced',
  created_at: '2026-07-20T00:00:00Z',
  updated_at: '2026-07-20T00:00:00Z'
}

describe('calendar response mapping', () => {
  it('maps a typed search result', () => {
    expect(mapCalendarSearchResponse({ results: [event] })).toEqual([event])
  })

  it('rejects an invalid search result instead of asserting it into the domain model', () => {
    expect(() => mapCalendarSearchResponse({ results: [{ event_id: 'event-1' }] }))
      .toThrow('Calendar search response contains an invalid event')
  })
})
