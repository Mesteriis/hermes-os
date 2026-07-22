import { describe, expect, it, vi } from 'vitest'
import { toggleCalendarService } from './integrationCalendarActions'
import type { CalendarAccount } from '../types/settings'

describe('integration calendar actions', () => {
  it('rejects a missing linked calendar account', async () => {
    const dependencies = dependenciesFor()

    await toggleCalendarService(null, true, dependencies)

    expect(dependencies.setError).toHaveBeenCalledWith(
      'No matching calendar account contract is available for this provider account.'
    )
    expect(dependencies.updateCalendarAccount).not.toHaveBeenCalled()
  })

  it('maps the service toggle to the calendar sync status contract', async () => {
    const dependencies = dependenciesFor()

    await toggleCalendarService(calendarAccount(), false, dependencies)

    expect(dependencies.updateCalendarAccount).toHaveBeenCalledWith({
      accountId: 'calendar-1',
      update: { sync_status: 'paused' }
    })
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('Calendar service paused')
    expect(dependencies.setActiveAccount).toHaveBeenLastCalledWith(null)
  })
})

function dependenciesFor() {
  return {
    t: (key: string) => key,
    setActiveAccount: vi.fn(),
    clearMessages: vi.fn(),
    setActionMessage: vi.fn(),
    setError: vi.fn(),
    updateCalendarAccount: vi.fn().mockResolvedValue({})
  }
}

function calendarAccount(): CalendarAccount {
  return {
    account_id: 'calendar-1',
    provider: 'google',
    account_name: 'Primary calendar',
    email: 'owner@example.com',
    credentials_reference: null,
    sync_status: 'active',
    capabilities: {},
    created_at: '2026-07-21T00:00:00Z',
    updated_at: '2026-07-21T00:00:00Z'
  }
}
