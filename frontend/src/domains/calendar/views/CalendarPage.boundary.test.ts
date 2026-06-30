import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('CalendarPage boundary', () => {
  it('preserves calendar orchestration after removing the CalendarPage Vue layer', () => {
    const surfaceSource = readFileSync(
      new URL('../queries/useCalendarPageSurface.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./CalendarPage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/CalendarToolbar.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/CalendarWeekGrid.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/CalendarUpcoming.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/CalendarSourceStatus.vue', import.meta.url))).toBe(false)
    expect(surfaceSource).toContain('useCalendarAccountsQuery')
    expect(surfaceSource).toContain('useCalendarEventsQuery')
    expect(surfaceSource).toContain('useCalendarSourcesQuery')
    expect(surfaceSource).toContain('useCalendarWeeklyBriefQuery')
    expect(surfaceSource).toContain('useCalendarEventBriefQuery')
    expect(surfaceSource).toContain('useCalendarEventAgendaQuery')
    expect(surfaceSource).toContain('useSearchCalendarEventsMutation')
    expect(surfaceSource).toContain('useCreateCalendarEventMutation')
    expect(surfaceSource).toContain('handleCreateEvent')
    expect(surfaceSource).toContain('handleRefreshAll')
    expect(surfaceSource).toContain('handleRefreshSelectedEvent')
    expect(surfaceSource).toContain('filteredEvents')
    expect(surfaceSource).not.toContain("from '../api/calendar'")
  })
})
