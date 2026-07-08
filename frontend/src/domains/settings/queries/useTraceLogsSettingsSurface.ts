import { computed, ref, watch } from 'vue'
import {
  useEventsQuery,
  useEventTraceByCorrelationIdQuery,
  useEventTraceByEventIdQuery,
  type EventTrace
} from '../../../platform/event-tracing'
import {
  buildRecentEventRows,
  buildTraceEventRows,
  buildTraceGraph,
  buildTraceSummaryTiles,
  filterRecentEventRows,
  filterTraceEventRows,
  paginateTraceRows,
  traceNodeDetail,
  type TraceDataTab,
  type TraceLookupMode
} from '../components/eventTraceSettingsPresentation'

export function useTraceLogsSettingsSurface() {
  const lookupMode = ref<TraceLookupMode>('event')
  const lookupInput = ref('')
  const submittedLookup = ref<{ mode: TraceLookupMode; id: string } | null>(null)
  const activeTraceDataTab = ref<TraceDataTab>('trace-events')
  const selectedTraceEventId = ref<string | null>(null)
  const traceLimit = ref(300)
  const traceEventSearch = ref('')
  const recentEventSearch = ref('')
  const traceEventPage = ref(1)
  const traceEventPageSize = ref(20)
  const recentEventsPageSize = ref(40)
  const recentEventsPageStarts = ref<number[]>([0])
  const recentEventsPageIndex = ref(0)
  const recentEventsAfterPosition = computed(() => recentEventsPageStarts.value[recentEventsPageIndex.value] ?? 0)
  const recentEventsQuery = useEventsQuery(recentEventsAfterPosition, recentEventsPageSize)

  const recentEvents = computed(() => recentEventsQuery.data.value?.items ?? [])
  const fallbackEventId = computed(() => recentEvents.value[0]?.event.event_id ?? null)
  const activeEventId = computed(() => {
    if (submittedLookup.value?.mode === 'event') return submittedLookup.value.id
    if (!submittedLookup.value) return fallbackEventId.value
    return null
  })
  const activeCorrelationId = computed(() =>
    submittedLookup.value?.mode === 'trace' ? submittedLookup.value.id : null
  )

  const traceByEventQuery = useEventTraceByEventIdQuery(activeEventId, traceLimit)
  const traceByCorrelationQuery = useEventTraceByCorrelationIdQuery(activeCorrelationId, traceLimit)
  const activeTrace = computed<EventTrace | null>(() => {
    if (activeCorrelationId.value) return traceByCorrelationQuery.data.value ?? null
    return traceByEventQuery.data.value ?? null
  })
  const activeTraceError = computed(() => {
    const query = activeCorrelationId.value ? traceByCorrelationQuery : traceByEventQuery
    if (!query.isError.value) return ''
    return query.error.value instanceof Error ? query.error.value.message : 'Trace request failed'
  })
  const isTraceLoading = computed(() =>
    activeCorrelationId.value ? traceByCorrelationQuery.isLoading.value : traceByEventQuery.isLoading.value
  )
  const isLoading = computed(() => recentEventsQuery.isLoading.value || isTraceLoading.value)
  const traceGraph = computed(() => buildTraceGraph(activeTrace.value))
  const traceEventRows = computed(() => buildTraceEventRows(activeTrace.value))
  const recentEventRows = computed(() => buildRecentEventRows(recentEvents.value))
  const filteredTraceEventRows = computed(() =>
    filterTraceEventRows(traceEventRows.value, traceEventSearch.value)
  )
  const filteredRecentEventRows = computed(() =>
    filterRecentEventRows(recentEventRows.value, recentEventSearch.value)
  )
  const pagedTraceEventRows = computed(() =>
    paginateTraceRows(filteredTraceEventRows.value, traceEventPage.value, traceEventPageSize.value)
  )
  const pagedRecentEventRows = computed(() =>
    paginateTraceRows(filteredRecentEventRows.value, 1, recentEventsPageSize.value)
  )
  const summaryTiles = computed(() => buildTraceSummaryTiles(activeTrace.value, recentEvents.value))
  const selectedNodeDetail = computed(() => traceNodeDetail(activeTrace.value, selectedTraceEventId.value))
  const activeTraceLabel = computed(() => {
    if (activeCorrelationId.value) return activeCorrelationId.value
    if (activeEventId.value) return activeEventId.value
    return 'No trace selected'
  })

  watch(activeTrace, (trace) => {
    selectedTraceEventId.value =
      trace?.root_event_ids.find((eventId) =>
        trace.events.some((stored) => stored.event.event_id === eventId)
      ) ??
      trace?.events[0]?.event.event_id ??
      null
  }, { immediate: true })

  watch(traceEventSearch, () => {
    traceEventPage.value = 1
  })

  watch(traceEventRows, () => {
    traceEventPage.value = 1
  })

  watch(recentEventSearch, () => {
    recentEventsPageIndex.value = 0
    recentEventsPageStarts.value = [0]
  })

  function handleLookupModeChange(mode: TraceLookupMode) {
    lookupMode.value = mode
  }

  function handleLookupInput(value: string) {
    lookupInput.value = value
  }

  function handleTraceDataTabChange(tab: TraceDataTab) {
    activeTraceDataTab.value = tab
  }

  function handleTraceEventSearch(value: string) {
    traceEventSearch.value = value
  }

  function handleRecentEventSearch(value: string) {
    recentEventSearch.value = value
  }

  function handleSubmitLookup() {
    const id = lookupInput.value.trim()
    if (!id) return
    submittedLookup.value = { mode: lookupMode.value, id }
  }

  function handleUseRecentEvent(eventId: string) {
    lookupMode.value = 'event'
    lookupInput.value = eventId
    submittedLookup.value = { mode: 'event', id: eventId }
  }

  function handleUseCorrelationId(correlationId: string) {
    lookupMode.value = 'trace'
    lookupInput.value = correlationId
    submittedLookup.value = { mode: 'trace', id: correlationId }
  }

  function handleSelectTraceNode(eventId: string) {
    selectedTraceEventId.value = eventId
  }

  function handleTraceEventsPreviousPage() {
    traceEventPage.value = Math.max(1, traceEventPage.value - 1)
  }

  function handleTraceEventsNextPage() {
    traceEventPage.value += 1
  }

  function handleRecentEventsPreviousPage() {
    if (recentEventsPageIndex.value <= 0) return
    recentEventsPageIndex.value -= 1
  }

  function handleRecentEventsNextPage() {
    const nextPosition = recentEventsQuery.data.value?.next_after_position
    if (!recentEventsQuery.data.value?.has_more || nextPosition === undefined) return
    const nextIndex = recentEventsPageIndex.value + 1
    recentEventsPageStarts.value = [
      ...recentEventsPageStarts.value.slice(0, nextIndex),
      nextPosition
    ]
    recentEventsPageIndex.value = nextIndex
  }

  function handleRefresh() {
    void recentEventsQuery.refetch()
    if (activeCorrelationId.value) {
      void traceByCorrelationQuery.refetch()
      return
    }
    void traceByEventQuery.refetch()
  }

  return {
    activeCorrelationId,
    activeEventId,
    activeTrace,
    activeTraceError,
    activeTraceLabel,
    activeTraceDataTab,
    filteredRecentEventRows,
    filteredTraceEventRows,
    handleRecentEventSearch,
    handleRecentEventsNextPage,
    handleRecentEventsPreviousPage,
    handleLookupInput,
    handleLookupModeChange,
    handleRefresh,
    handleSelectTraceNode,
    handleSubmitLookup,
    handleTraceDataTabChange,
    handleTraceEventSearch,
    handleTraceEventsNextPage,
    handleTraceEventsPreviousPage,
    handleUseCorrelationId,
    handleUseRecentEvent,
    isLoading,
    isTraceLoading,
    lookupInput,
    lookupMode,
    pagedRecentEventRows,
    pagedTraceEventRows,
    recentEventSearch,
    recentEventRows,
    recentEventsHasMore: computed(() => recentEventsQuery.data.value?.has_more ?? false),
    recentEventsIsFetching: recentEventsQuery.isFetching,
    recentEventsPage: computed(() => recentEventsPageIndex.value + 1),
    recentEventsPageSize,
    selectedNodeDetail,
    selectedTraceEventId,
    summaryTiles,
    traceEventPage,
    traceEventPageSize,
    traceEventSearch,
    traceEventRows,
    traceGraph,
    traceLimit
  }
}

export type TraceLogsSettingsSurface = ReturnType<typeof useTraceLogsSettingsSurface>
