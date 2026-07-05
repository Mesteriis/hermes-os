import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/calendar/queries/useCalendarPageSurface.ts'

export function useCalendarSurface() {
  return createDomainSurface({
    surfaceId: 'calendar',
    labelKey: 'Calendar',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'calendar-agenda',
        labelKey: 'Agenda',
        descriptionKey: 'Calendar events, selected date state and schedule context.',
        icon: 'tabler:calendar-event',
        status: 'active',
        kind: 'query',
        contract: 'useCalendarPageSurface.calendarEvents'
      },
      {
        id: 'calendar-create-event',
        labelKey: 'Create event',
        descriptionKey: 'Event creation flow with provider-backed account boundaries.',
        icon: 'tabler:calendar-plus',
        status: 'active',
        kind: 'command',
        contract: 'useCalendarPageSurface.createEvent'
      },
      {
        id: 'calendar-brief',
        labelKey: 'Calendar brief',
        descriptionKey: 'Daily and weekly planning context for owner review.',
        icon: 'tabler:calendar-stats',
        status: 'active',
        kind: 'projection',
        contract: 'useCalendarPageSurface.weeklyBrief'
      }
    ],
    childSurfaces: [
      {
        id: 'calendar-agenda',
        labelKey: 'Agenda',
        status: 'facade',
        surfacePath,
        capabilityIds: ['calendar-agenda']
      },
      {
        id: 'calendar-create',
        labelKey: 'Create',
        status: 'facade',
        surfacePath,
        capabilityIds: ['calendar-create-event']
      },
      {
        id: 'calendar-brief',
        labelKey: 'Brief',
        status: 'facade',
        surfacePath,
        capabilityIds: ['calendar-brief']
      }
    ]
  })
}

