import { computed, ref } from 'vue'
import {
  useCalendarEventAgendaQuery,
  useCalendarEventBriefQuery,
  useCalendarEventContextPackQuery,
  useCalendarEventsQuery,
  useCalendarAccountsQuery,
  useCalendarSourcesQuery,
  useCalendarWeeklyBriefQuery,
  useCreateCalendarEventMutation,
  useSearchCalendarEventsMutation,
} from './useCalendarEventsQuery'
import {
  EVENT_TYPE_OPTIONS,
  filterWeekEvents,
  getWeekColumns,
  getWeekStart,
  useCalendarStore,
} from '../stores/calendar'
import type { CalendarEvent } from '../types/calendar'

export function useCalendarPageSurface() {
  const store = useCalendarStore()

  const accountsQuery = useCalendarAccountsQuery()
  const eventsQuery = useCalendarEventsQuery(200)
  const createCalendarEventMutation = useCreateCalendarEventMutation()
  const searchCalendarEventsMutation = useSearchCalendarEventsMutation()

  const calendarAccounts = computed(() => accountsQuery.data.value ?? [])
  const calendarEvents = computed(() => eventsQuery.data.value ?? [])
  const calendarAccountIds = computed(() => calendarAccounts.value.map((account) => account.account_id))
  const sourcesQuery = useCalendarSourcesQuery(calendarAccountIds)
  const weeklyBriefQuery = useCalendarWeeklyBriefQuery()

  const searchResults = ref<CalendarEvent[]>([])
  const weekStart = ref(getWeekStart())
  const weekColumns = computed(() => getWeekColumns(weekStart.value))
  const filteredEvents = computed(() => filterWeekEvents(calendarEvents.value, weekStart.value))

  const isLoading = computed(() => accountsQuery.isLoading.value || eventsQuery.isLoading.value)
  const displayEvents = computed(() =>
    searchResults.value.length > 0 ? searchResults.value : filteredEvents.value
  )
  const selectedEventId = computed(() => store.selectedEvent?.event_id ?? null)
  const eventContextQuery = useCalendarEventContextPackQuery(selectedEventId)
  const eventBriefQuery = useCalendarEventBriefQuery(selectedEventId)
  const eventAgendaQuery = useCalendarEventAgendaQuery(selectedEventId)

  const calendarSources = computed(() => sourcesQuery.data.value ?? [])
  const weeklyBrief = computed(() => weeklyBriefQuery.data.value ?? null)
  const eventContext = computed(() => eventContextQuery.data.value ?? null)
  const eventBrief = computed(() => eventBriefQuery.data.value ?? null)
  const eventAgenda = computed(() => eventAgendaQuery.data.value ?? null)

  async function handleSearch() {
    if (!store.searchQuery.trim()) {
      searchResults.value = []
      return
    }
    try {
      searchResults.value = await searchCalendarEventsMutation.mutateAsync(store.searchQuery)
    } catch {
      searchResults.value = []
    }
  }

  function handlePrepareEvent(event: CalendarEvent) {
    store.selectEvent(event)
  }

  async function handleCreateEvent() {
    if (!store.newEventTitle || !store.newEventStart || !store.newEventEnd) return
    try {
      await createCalendarEventMutation.mutateAsync({
        title: store.newEventTitle,
        start_at: new Date(store.newEventStart).toISOString(),
        end_at: new Date(store.newEventEnd).toISOString(),
        event_type: store.newEventType,
      })
      store.resetNewEventForm()
      await eventsQuery.refetch()
    } catch (error) {
      store.setCalendarError(error instanceof Error ? error.message : 'Create failed')
    }
  }

  async function handleRefreshAll() {
    await Promise.all([
      eventsQuery.refetch(),
      sourcesQuery.refetch(),
      weeklyBriefQuery.refetch(),
    ])
  }

  async function handleRefreshWeeklyBrief() {
    await weeklyBriefQuery.refetch()
  }

  async function handleRefreshSelectedEvent() {
    await Promise.all([
      eventContextQuery.refetch(),
      eventBriefQuery.refetch(),
      eventAgendaQuery.refetch(),
    ])
  }

  return {
    EVENT_TYPE_OPTIONS,
    calendarAccounts,
    calendarEvents,
    calendarSources,
    createCalendarEventMutation,
    displayEvents,
    eventAgenda,
    eventBrief,
    eventContext,
    filteredEvents,
    handleCreateEvent,
    handlePrepareEvent,
    handleRefreshAll,
    handleRefreshSelectedEvent,
    handleRefreshWeeklyBrief,
    handleSearch,
    isLoading,
    searchResults,
    store,
    weekColumns,
    weekStart,
    weeklyBrief,
  }
}
