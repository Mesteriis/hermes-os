import type { TraceLogsSettingsSurface } from './useTraceLogsSettingsSurface'

export function useTraceLogsSettingsPanelController(options: { surface: TraceLogsSettingsSurface }) {
  const surface = options.surface

  function eventValue(event: Event): string {
    return event.target instanceof HTMLInputElement
      || event.target instanceof HTMLSelectElement
      || event.target instanceof HTMLTextAreaElement
      ? event.target.value
      : ''
  }

  function handleLookupInput(event: Event): void {
    surface.handleLookupInput(eventValue(event))
  }

  function handleTraceEventSearch(event: Event): void {
    surface.handleTraceEventSearch(eventValue(event))
  }

  function handleRecentEventSearch(event: Event): void {
    surface.handleRecentEventSearch(eventValue(event))
  }

  return {
    handleLookupInput,
    handleTraceEventSearch,
    handleRecentEventSearch,
    handleLookupModeChange: surface.handleLookupModeChange,
    handleRefresh: surface.handleRefresh,
    handleSelectTraceNode: surface.handleSelectTraceNode,
    handleTraceDataTabChange: surface.handleTraceDataTabChange,
    handleUseRecentEvent: surface.handleUseRecentEvent,
    handleTraceEventsPreviousPage: surface.handleTraceEventsPreviousPage,
    handleTraceEventsNextPage: surface.handleTraceEventsNextPage,
    handleRecentEventsPreviousPage: surface.handleRecentEventsPreviousPage,
    handleRecentEventsNextPage: surface.handleRecentEventsNextPage,
    handleSubmitLookup: surface.handleSubmitLookup,
  }
}
