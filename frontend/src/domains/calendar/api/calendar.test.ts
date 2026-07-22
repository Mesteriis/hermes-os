import { describe, expect, it } from 'vitest'
import { mapWeeklyBrief } from './calendar'

describe('calendar API mapping', () => {
  it('maps the required weekly brief counters', () => {
    expect(mapWeeklyBrief({
      upcoming_events_this_week: 3,
      overdue_deadlines: 1,
      past_events_without_notes: 2,
      generated_at: '2026-07-21T00:00:00Z'
    })).toEqual({
      upcoming_events_this_week: 3,
      overdue_deadlines: 1,
      past_events_without_notes: 2,
      generated_at: '2026-07-21T00:00:00Z'
    })
  })

  it('rejects a weekly brief with invalid required counters', () => {
    expect(() => mapWeeklyBrief({
      upcoming_events_this_week: '3',
      overdue_deadlines: 1,
      past_events_without_notes: 2
    })).toThrow('Calendar weekly brief has invalid required counters')
  })
})
