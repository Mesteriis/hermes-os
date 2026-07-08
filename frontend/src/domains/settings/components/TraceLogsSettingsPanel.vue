<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { TraceLogsSettingsSurface } from '../queries/useTraceLogsSettingsSurface'
import type { TraceDataTab, TraceLookupMode } from './eventTraceSettingsPresentation'

defineProps<{
  surface: TraceLogsSettingsSurface
}>()

const { t } = useI18n()

const lookupModes: Array<{ id: TraceLookupMode; label: string; icon: string }> = [
  { id: 'event', label: 'Event ID', icon: 'tabler:timeline-event' },
  { id: 'trace', label: 'Trace ID', icon: 'tabler:route' }
]

const traceDataTabs: Array<{ id: TraceDataTab; label: string; icon: string }> = [
  { id: 'trace-events', label: 'Trace events', icon: 'tabler:timeline' },
  { id: 'recent-seeds', label: 'Event log seeds', icon: 'tabler:list-search' }
]

function eventValue(event: Event): string {
  return event.target instanceof HTMLInputElement ? event.target.value : ''
}
</script>

<template>
  <section class="settings-section settings-trace-section">
    <header class="settings-section-toolbar">
      <div>
        <h3>{{ t('Logs & Traces') }}</h3>
        <p>{{ t('Causal event logs, span graph and trace annotations from the platform event store.') }}</p>
      </div>
      <button
        type="button"
        class="icon-button"
        :title="t('Refresh traces')"
        :aria-label="t('Refresh traces')"
        @click="surface.handleRefresh()"
      >
        <Icon icon="tabler:refresh" />
      </button>
    </header>

    <section class="settings-trace-summary" :aria-label="t('Trace summary')">
      <article
        v-for="tile in surface.summaryTiles.value"
        :key="tile.id"
        class="settings-trace-summary-tile"
        :class="`tone-${tile.tone}`"
      >
        <Icon :icon="tile.icon" />
        <span>{{ t(tile.label) }}</span>
        <strong>{{ tile.value }}</strong>
        <small>{{ t(tile.detail) }}</small>
      </article>
    </section>

    <section class="settings-trace-panel" :aria-label="t('Trace lookup')">
      <header class="settings-trace-panel__header">
        <div>
          <span>{{ t('Lookup') }}</span>
          <strong>{{ t('Find event trace') }}</strong>
        </div>
        <small>{{ t('Event envelopes are rendered as spans; correlation_id is the trace id.') }}</small>
      </header>

      <form class="settings-trace-lookup" @submit.prevent="surface.handleSubmitLookup()">
        <div class="settings-trace-mode-tabs" :aria-label="t('Lookup mode')">
          <button
            v-for="mode in lookupModes"
            :key="mode.id"
            type="button"
            class="settings-trace-mode-tab"
            :class="{ active: surface.lookupMode.value === mode.id }"
            :aria-pressed="surface.lookupMode.value === mode.id"
            @click="surface.handleLookupModeChange(mode.id)"
          >
            <Icon :icon="mode.icon" />
            <span>{{ t(mode.label) }}</span>
          </button>
        </div>
        <input
          type="text"
          :value="surface.lookupInput.value"
          :placeholder="surface.lookupMode.value === 'event' ? t('Paste event_id') : t('Paste correlation_id')"
          autocomplete="off"
          @input="surface.handleLookupInput(eventValue($event))"
        >
        <button type="submit" class="primary-button">
          <Icon icon="tabler:search" />
          {{ t('Open trace') }}
        </button>
      </form>
    </section>

    <section class="settings-trace-panel settings-trace-graph-panel" :aria-label="t('Trace graph')">
      <header class="settings-trace-panel__header">
        <div>
          <span>{{ t('Graph') }}</span>
          <strong>{{ surface.activeTraceLabel.value }}</strong>
        </div>
        <small>{{ t('Causation edges connect parent span ids to child event envelopes.') }}</small>
      </header>

      <div v-if="surface.isTraceLoading.value" class="settings-empty-state">
        <Icon icon="tabler:loader-2" />
        <strong>{{ t('Loading trace') }}</strong>
      </div>

      <div v-else-if="surface.activeTraceError.value" class="settings-empty-state">
        <Icon icon="tabler:alert-circle" />
        <strong>{{ t('Trace unavailable') }}</strong>
        <span>{{ surface.activeTraceError.value }}</span>
      </div>

      <div v-else class="settings-trace-graph-layout">
        <div class="settings-trace-graph-scroll">
          <svg
            class="settings-trace-graph-svg"
            :viewBox="`0 0 ${surface.traceGraph.value.width} ${surface.traceGraph.value.height}`"
            role="img"
            :aria-label="t('Trace causation graph')"
          >
            <line
              v-for="edge in surface.traceGraph.value.edges"
              :key="edge.id"
              class="settings-trace-edge"
              :class="`tone-${edge.tone}`"
              :x1="edge.x1"
              :y1="edge.y1"
              :x2="edge.x2"
              :y2="edge.y2"
            />
            <g
              v-for="node in surface.traceGraph.value.nodes"
              :key="node.eventId"
              class="settings-trace-node"
              :class="[`tone-${node.tone}`, { active: surface.selectedTraceEventId.value === node.eventId }]"
              :transform="`translate(${node.x} ${node.y})`"
              role="button"
              tabindex="0"
              @click="surface.handleSelectTraceNode(node.eventId)"
              @keydown.enter.prevent="surface.handleSelectTraceNode(node.eventId)"
              @keydown.space.prevent="surface.handleSelectTraceNode(node.eventId)"
            >
              <rect :width="node.width" :height="node.height" rx="8" />
              <text class="settings-trace-node__title" x="14" y="25">{{ node.title }}</text>
              <text class="settings-trace-node__subtitle" x="14" y="49">{{ node.subtitle }}</text>
              <text class="settings-trace-node__meta" x="14" y="74">{{ node.meta }}</text>
            </g>
          </svg>
        </div>

        <aside class="settings-trace-node-detail" :aria-label="t('Selected span')">
          <template v-if="surface.selectedNodeDetail.value">
            <span>{{ t('Selected span') }}</span>
            <strong>{{ surface.selectedNodeDetail.value.eventType }}</strong>
            <dl>
              <div>
                <dt>{{ t('event_id') }}</dt>
                <dd>{{ surface.selectedNodeDetail.value.eventId }}</dd>
              </div>
              <div>
                <dt>{{ t('position') }}</dt>
                <dd>{{ surface.selectedNodeDetail.value.position }}</dd>
              </div>
              <div>
                <dt>{{ t('recorded_at') }}</dt>
                <dd>{{ surface.selectedNodeDetail.value.recordedAt }}</dd>
              </div>
              <div>
                <dt>{{ t('causation_id') }}</dt>
                <dd>{{ surface.selectedNodeDetail.value.causationId }}</dd>
              </div>
              <div>
                <dt>{{ t('correlation_id') }}</dt>
                <dd>{{ surface.selectedNodeDetail.value.correlationId }}</dd>
              </div>
              <div>
                <dt>{{ t('source') }}</dt>
                <dd>{{ surface.selectedNodeDetail.value.sourceLabel }}</dd>
              </div>
              <div>
                <dt>{{ t('subject') }}</dt>
                <dd>{{ surface.selectedNodeDetail.value.subjectLabel }}</dd>
              </div>
              <div>
                <dt>{{ t('consumers') }}</dt>
                <dd>{{ surface.selectedNodeDetail.value.consumerLabel }}</dd>
              </div>
              <div>
                <dt>{{ t('dead letters') }}</dt>
                <dd>{{ surface.selectedNodeDetail.value.deadLetterLabel }}</dd>
              </div>
            </dl>
          </template>
          <template v-else>
            <Icon icon="tabler:route-off" />
            <strong>{{ t('No span selected') }}</strong>
          </template>
        </aside>
      </div>
    </section>

    <section class="settings-trace-panel settings-trace-data-panel" :aria-label="t('Trace event data')">
      <header class="settings-trace-panel__header settings-trace-panel__header--compact">
        <div>
          <span>{{ t(surface.activeTraceDataTab.value === 'trace-events' ? 'Timeline' : 'Event log') }}</span>
          <strong>{{ t(surface.activeTraceDataTab.value === 'trace-events' ? 'Events in trace' : 'Recent seeds') }}</strong>
        </div>
        <small>
          {{ t(surface.activeTraceDataTab.value === 'trace-events'
            ? 'Payload bodies stay sanitized by the platform event API.'
            : 'Use a seed event to open its full causal trace.') }}
        </small>
      </header>

      <div class="settings-trace-data-toolbar">
        <nav class="settings-trace-data-tabs" :aria-label="t('Trace data views')">
          <button
            v-for="tab in traceDataTabs"
            :key="tab.id"
            type="button"
            class="settings-trace-data-tab"
            :class="{ active: surface.activeTraceDataTab.value === tab.id }"
            :aria-pressed="surface.activeTraceDataTab.value === tab.id"
            @click="surface.handleTraceDataTabChange(tab.id)"
          >
            <Icon :icon="tab.icon" />
            <span>{{ t(tab.label) }}</span>
            <strong>
              {{ tab.id === 'trace-events'
                ? surface.filteredTraceEventRows.value.length
                : surface.filteredRecentEventRows.value.length }}
            </strong>
          </button>
        </nav>

        <label class="settings-trace-data-search">
          <Icon icon="tabler:search" />
          <input
            v-if="surface.activeTraceDataTab.value === 'trace-events'"
            type="search"
            :value="surface.traceEventSearch.value"
            :placeholder="t('Search events in trace')"
            :aria-label="t('Search events in trace')"
            autocomplete="off"
            @input="surface.handleTraceEventSearch(eventValue($event))"
          >
          <input
            v-else
            type="search"
            :value="surface.recentEventSearch.value"
            :placeholder="t('Search loaded event seeds')"
            :aria-label="t('Search loaded event seeds')"
            autocomplete="off"
            @input="surface.handleRecentEventSearch(eventValue($event))"
          >
        </label>
      </div>

      <template v-if="surface.activeTraceDataTab.value === 'trace-events'">
        <div
          v-if="surface.pagedTraceEventRows.value.rows.length === 0"
          class="settings-empty-state"
        >
          <Icon icon="tabler:timeline-off" />
          <strong>{{ t('No trace events found') }}</strong>
        </div>

        <div v-else class="settings-trace-table-scroll">
          <table class="settings-trace-table">
            <thead>
              <tr>
                <th scope="col">{{ t('Position') }}</th>
                <th scope="col">{{ t('Event') }}</th>
                <th scope="col">{{ t('Trace') }}</th>
                <th scope="col">{{ t('Source') }}</th>
                <th scope="col">{{ t('Recorded') }}</th>
                <th scope="col">{{ t('Annotations') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="row in surface.pagedTraceEventRows.value.rows"
                :key="row.eventId"
                :class="`tone-${row.tone}`"
                @click="surface.handleSelectTraceNode(row.eventId)"
              >
                <td><strong>#{{ row.position }}</strong></td>
                <td>
                  <strong>{{ row.eventType }}</strong>
                  <code>{{ row.eventId }}</code>
                </td>
                <td>
                  <small>{{ row.correlationId }}</small>
                  <small>{{ row.causationId }}</small>
                </td>
                <td>
                  <span>{{ row.sourceLabel }}</span>
                  <small>{{ row.subjectLabel }}</small>
                </td>
                <td>{{ row.recordedAt }}</td>
                <td>{{ row.annotationLabel }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <footer class="settings-trace-pagination">
          <span>
            {{ surface.pagedTraceEventRows.value.startIndex }}-{{ surface.pagedTraceEventRows.value.endIndex }}
            / {{ surface.pagedTraceEventRows.value.totalCount }}
          </span>
          <div>
            <button
              type="button"
              class="icon-button"
              :title="t('Previous page')"
              :aria-label="t('Previous page')"
              :disabled="!surface.pagedTraceEventRows.value.hasPrevious"
              @click="surface.handleTraceEventsPreviousPage()"
            >
              <Icon icon="tabler:chevron-left" />
            </button>
            <strong>
              {{ surface.pagedTraceEventRows.value.page }} / {{ surface.pagedTraceEventRows.value.pageCount }}
            </strong>
            <button
              type="button"
              class="icon-button"
              :title="t('Next page')"
              :aria-label="t('Next page')"
              :disabled="!surface.pagedTraceEventRows.value.hasNext"
              @click="surface.handleTraceEventsNextPage()"
            >
              <Icon icon="tabler:chevron-right" />
            </button>
          </div>
        </footer>
      </template>

      <template v-else>
        <div
          v-if="surface.pagedRecentEventRows.value.rows.length === 0"
          class="settings-empty-state"
        >
          <Icon icon="tabler:list-search" />
          <strong>{{ t('No event seeds found') }}</strong>
        </div>

        <div v-else class="settings-trace-seed-list">
          <button
            v-for="row in surface.pagedRecentEventRows.value.rows"
            :key="row.eventId"
            type="button"
            class="settings-trace-seed"
            @click="surface.handleUseRecentEvent(row.eventId)"
          >
            <span>
              <strong>{{ row.eventType }}</strong>
              <small>#{{ row.position }} · {{ row.recordedAt }}</small>
            </span>
            <code>{{ row.correlationId }}</code>
            <Icon icon="tabler:arrow-right" />
          </button>
        </div>

        <footer class="settings-trace-pagination">
          <span>
            {{ surface.pagedRecentEventRows.value.totalCount }} {{ t('loaded') }}
          </span>
          <div>
            <button
              type="button"
              class="icon-button"
              :title="t('Previous page')"
              :aria-label="t('Previous page')"
              :disabled="surface.recentEventsPage.value <= 1"
              @click="surface.handleRecentEventsPreviousPage()"
            >
              <Icon icon="tabler:chevron-left" />
            </button>
            <strong>{{ surface.recentEventsPage.value }}</strong>
            <button
              type="button"
              class="icon-button"
              :title="t('Load next page')"
              :aria-label="t('Load next page')"
              :disabled="!surface.recentEventsHasMore.value || surface.recentEventsIsFetching.value"
              @click="surface.handleRecentEventsNextPage()"
            >
              <Icon icon="tabler:chevron-right" />
            </button>
          </div>
        </footer>
      </template>
    </section>
  </section>
</template>
