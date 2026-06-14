import { defineStore } from 'pinia'
import { ref } from 'vue'
import { format, formatDistanceToNow } from 'date-fns'
import type { CalendarEvent, CalendarViewMode, WeeklyBrief } from '../types/calendar'

export const useCalendarStore = defineStore('calendar-ui', () => {
  const viewMode = ref<CalendarViewMode>('week')
  const searchQuery = ref('')
  const selectedEvent = ref<CalendarEvent | null>(null)
  const calendarError = ref('')
  const isCalendarLoading = ref(false)
  const showNewEventForm = ref(false)
  const newEventTitle = ref('')
  const newEventStart = ref('')
  const newEventEnd = ref('')
  const newEventType = ref('meeting')
  const weeklyBrief = ref<WeeklyBrief | null>(null)
  const eventBrief = ref<Record<string, unknown> | null>(null)
  const eventAgenda = ref<Record<string, unknown> | null>(null)

  function setViewMode(mode: CalendarViewMode) {
    viewMode.value = mode
  }

  function setSearchQuery(query: string) {
    searchQuery.value = query
  }

  function selectEvent(evt: CalendarEvent | null) {
    selectedEvent.value = evt
    if (!evt) {
      eventBrief.value = null
      eventAgenda.value = null
    }
  }

  function setCalendarError(error: string) {
    calendarError.value = error
  }

  function setCalendarLoading(loading: boolean) {
    isCalendarLoading.value = loading
  }

  function toggleNewEventForm() {
    showNewEventForm.value = !showNewEventForm.value
  }

  function resetNewEventForm() {
    newEventTitle.value = ''
    newEventStart.value = ''
    newEventEnd.value = ''
    newEventType.value = 'meeting'
    showNewEventForm.value = false
  }

  function setWeeklyBrief(brief: WeeklyBrief | null) {
    weeklyBrief.value = brief
  }

  function setEventBrief(brief: Record<string, unknown> | null) {
    eventBrief.value = brief
  }

  function setEventAgenda(agenda: Record<string, unknown> | null) {
    eventAgenda.value = agenda
  }

  return {
    viewMode, searchQuery, selectedEvent, calendarError, isCalendarLoading,
    showNewEventForm, newEventTitle, newEventStart, newEventEnd, newEventType,
    weeklyBrief, eventBrief, eventAgenda,
    setViewMode, setSearchQuery, selectEvent, setCalendarError, setCalendarLoading,
    toggleNewEventForm, resetNewEventForm,
    setWeeklyBrief, setEventBrief, setEventAgenda
  }
})

// --- Utility functions ---

export function formatEventDate(dateStr: string): string {
  const d = new Date(dateStr)
  return format(d, 'EEE, MMM d')
}

export function formatEventTime(dateStr: string): string {
  const d = new Date(dateStr)
  return format(d, 'HH:mm')
}

export function formatEventDateTime(dateStr: string): string {
  const d = new Date(dateStr)
  return format(d, 'EEE, MMM d HH:mm')
}

export function formatEventDayShort(dateStr: string): string {
  const d = new Date(dateStr)
  return format(d, 'EEE d')
}

export function formatRelativeTime(dateStr: string): string {
  const d = new Date(dateStr)
  const now = new Date()
  if (d < now) return `${formatDistanceToNow(d)} ago`
  return formatDistanceToNow(d, { addSuffix: true })
}

export function eventTypeTone(eventType: string | null): string {
  switch (eventType) {
    case 'meeting': return 'blue'
    case 'deadline': return 'red'
    case 'focus': return 'green'
    default: return 'neutral'
  }
}

export function eventTypeLabel(type: string): string {
  return type.charAt(0).toUpperCase() + type.slice(1)
}

export const WEEK_DAYS = ['MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT', 'SUN']

export const EVENT_TYPE_OPTIONS = [
  'meeting', 'focus', 'deadline', 'personal', 'travel', 'tax', 'review', 'planning'
] as const

export function getWeekStart(): Date {
  const now = new Date()
  const start = new Date(now)
  start.setDate(now.getDate() - now.getDay() + 1)
  start.setHours(0, 0, 0, 0)
  return start
}

export function getWeekEnd(weekStart: Date): Date {
  const end = new Date(weekStart)
  end.setDate(weekStart.getDate() + 7)
  return end
}

export function getWeekColumns(weekStart: Date): string[] {
  return WEEK_DAYS.map((d, i) => {
    const d2 = new Date(weekStart)
    d2.setDate(weekStart.getDate() + i)
    return `${d} ${d2.getDate()}`
  })
}

export function filterWeekEvents(events: CalendarEvent[], weekStart: Date): CalendarEvent[] {
  const end = getWeekEnd(weekStart)
  return events.filter(e => {
    const start = new Date(e.start_at)
    return start >= weekStart && start < end
  })
}
