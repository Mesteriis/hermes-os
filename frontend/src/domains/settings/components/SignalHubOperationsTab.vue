<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import { useI18n } from '../../../platform/i18n'
import type { SignalHubSettingsController } from './useSignalHubSettingsController'
import {
  connectionLabel,
  formatConnectionTimeline,
  formatHealthEvidence,
  formatHealthStatus,
  formatRuntimeError,
  formatRuntimeTimeline,
  formatSettingsSummary,
  healthTone,
  runtimeTone,
  sourceIconForCode,
  statusTone
} from './signalHubSettingsPresentation'

const props = defineProps<{ state: SignalHubSettingsController }>()
const { t } = useI18n()

const sources = computed(() => props.state.sources.value)
const connections = computed(() => props.state.connections.value)
const runtimeStates = computed(() => props.state.runtimeStates.value)
const healthItems = computed(() => props.state.healthItems.value)
const replayRequests = computed(() => props.state.replayRequests.value)
const connectionCapableSources = computed(() => props.state.connectionCapableSources.value)
const replayScopedConnections = computed(() => props.state.replayScopedConnections.value)
const replayTargetConsumers = computed(() => props.state.replayTargetConsumers.value)
</script>

<template>
  <div v-if="state.activeTab.value === 'connections'" class="signal-table-layout">
    <div class="policy-layout">
      <form class="policy-form" @submit.prevent="state.handleCreateConnection">
        <label>
          <span>{{ t('Source') }}</span>
          <select v-model="state.connectionSourceCode.value" class="hermes-select-control">
            <option v-for="source in connectionCapableSources" :key="source.code" :value="source.code">
              {{ source.display_name }}
            </option>
          </select>
        </label>
        <label>
          <span>{{ t('Display Name') }}</span>
          <input v-model="state.connectionDisplayName.value" class="hermes-input-control" type="text" :placeholder="t('Connection display name')" />
        </label>
        <label>
          <span>{{ t('Profile') }}</span>
          <input v-model="state.connectionProfile.value" class="hermes-input-control" type="text" :placeholder="t('Connection profile')" />
        </label>
        <button type="submit" class="hermes-btn" :disabled="state.isCreatingConnection.value">
          <Icon icon="tabler:plug-connected" />
          {{ state.isCreatingConnection.value ? t('Creating') : t('Create Connection') }}
        </button>
      </form>

      <div v-if="connections.length === 0" class="empty-panel fill">{{ t('No source connections yet.') }}</div>
      <div v-else class="signal-table">
        <div v-for="connection in connections" :key="connection.id" class="signal-table-row signal-runtime-row">
          <div class="signal-table-main">
            <Icon :icon="sourceIconForCode(sources, connection.source_code)" />
            <span>
              <strong>{{ connection.display_name }}</strong>
              <em>{{ connection.source_code }} / {{ connection.profile ?? t('No profile') }}</em>
              <small class="signal-detail-text">{{ formatSettingsSummary(t, connection) }}</small>
              <small class="signal-detail-text">{{ formatConnectionTimeline(t, connection) }}</small>
            </span>
          </div>
          <b class="signal-pill" :data-tone="statusTone(connection.status)">{{ connection.status }}</b>
          <div class="runtime-actions">
            <button type="button" class="hermes-btn hermes-btn--outline hermes-btn--compact" :disabled="state.isUpdatingConnection.value || connection.status === 'connected'" @click="state.handleSetConnectionStatus(connection.id, 'connected')">{{ t('Connect') }}</button>
            <button type="button" class="hermes-btn hermes-btn--outline hermes-btn--compact" :disabled="state.isUpdatingConnection.value || connection.status === 'paused'" @click="state.handleSetConnectionStatus(connection.id, 'paused')">{{ t('Pause') }}</button>
            <button type="button" class="hermes-btn hermes-btn--outline hermes-btn--compact" :disabled="state.isUpdatingConnection.value || connection.status === 'muted'" @click="state.handleSetConnectionStatus(connection.id, 'muted')">{{ t('Mute') }}</button>
            <button type="button" class="hermes-btn hermes-btn--outline hermes-btn--compact" :disabled="state.isUpdatingConnection.value || connection.status === 'disabled'" @click="state.handleSetConnectionStatus(connection.id, 'disabled')">{{ t('Disable') }}</button>
            <button type="button" class="hermes-btn hermes-btn--outline hermes-btn--compact" :disabled="state.isUpdatingConnection.value || connection.status === 'removed'" @click="state.handleRemoveConnection(connection.id)">{{ t('Remove') }}</button>
          </div>
        </div>
      </div>
    </div>
  </div>

  <div v-else-if="state.activeTab.value === 'runtime'" class="signal-table-layout">
    <div v-if="runtimeStates.length === 0" class="empty-panel fill">{{ t('No runtime controls yet.') }}</div>
    <div v-else class="signal-table">
      <div v-for="runtime in runtimeStates" :key="runtime.id" class="signal-table-row signal-runtime-row">
        <div class="signal-table-main">
          <Icon :icon="sourceIconForCode(sources, runtime.source_code)" />
          <span>
            <strong>{{ runtime.runtime_kind }}</strong>
            <em>{{ runtime.source_code }}</em>
            <small class="signal-detail-text">{{ formatRuntimeTimeline(t, runtime) }}</small>
            <small v-if="formatRuntimeError(t, runtime)" class="signal-detail-text signal-detail-text--bad">{{ formatRuntimeError(t, runtime) }}</small>
          </span>
        </div>
        <b class="signal-pill" :data-tone="runtimeTone(runtime.state)">{{ runtime.state }}</b>
        <div class="runtime-actions">
          <button type="button" class="hermes-btn hermes-btn--outline hermes-btn--compact" :disabled="state.isUpdatingRuntime.value || runtime.state === 'running'" @click="state.handleSetRuntimeState(runtime, 'running')">{{ t('Run') }}</button>
          <button type="button" class="hermes-btn hermes-btn--outline hermes-btn--compact" :disabled="state.isUpdatingRuntime.value || runtime.state === 'paused'" @click="state.handleSetRuntimeState(runtime, 'paused')">{{ t('Pause') }}</button>
          <button type="button" class="hermes-btn hermes-btn--outline hermes-btn--compact" :disabled="state.isUpdatingRuntime.value || runtime.state === 'muted'" @click="state.handleSetRuntimeState(runtime, 'muted')">{{ t('Mute') }}</button>
          <button type="button" class="hermes-btn hermes-btn--outline hermes-btn--compact" :disabled="state.isUpdatingRuntime.value || runtime.state === 'stopped'" @click="state.handleSetRuntimeState(runtime, 'stopped')">{{ t('Stop') }}</button>
        </div>
      </div>
    </div>
  </div>

  <div v-else-if="state.activeTab.value === 'health'" class="signal-table-layout">
    <div v-if="healthItems.length === 0" class="empty-panel fill">{{ t('No health records yet.') }}</div>
    <div v-else class="signal-table">
      <div v-for="item in healthItems" :key="item.id" class="signal-table-row">
        <div class="signal-table-main">
          <Icon :icon="sourceIconForCode(sources, item.source_code)" />
          <span>
            <strong>{{ item.summary }}</strong>
            <em>{{ item.source_code }}</em>
            <small class="signal-detail-text">{{ formatHealthStatus(t, connections, item) }}</small>
            <small v-if="formatHealthEvidence(t, item)" class="signal-detail-text">{{ formatHealthEvidence(t, item) }}</small>
          </span>
        </div>
        <b class="signal-pill" :data-tone="healthTone(item.level)">{{ item.level }}</b>
        <div class="runtime-actions">
          <small>{{ item.next_retry_at ? `${t('Retry')} ${item.next_retry_at}` : item.last_ok_at ?? item.last_failure_at ?? t('No heartbeat') }}</small>
          <button type="button" class="hermes-btn hermes-btn--outline hermes-btn--compact" :disabled="state.isRunningHealthCheck.value" @click="state.handleRunHealthCheck(item.source_code, item.connection_id)">
            {{ state.isRunningHealthCheck.value ? t('Checking') : t('Run Check') }}
          </button>
        </div>
      </div>
    </div>
  </div>

  <div v-else-if="state.activeTab.value === 'replay'" class="signal-table-layout">
    <div class="policy-layout">
      <form class="policy-form" @submit.prevent="state.handleCreateReplayRequest">
        <label>
          <span>{{ t('Source') }}</span>
          <select v-model="state.replaySourceCode.value" class="hermes-select-control">
            <option value="">{{ t('All sources') }}</option>
            <option v-for="source in sources.filter((item) => item.supports_replay)" :key="source.code" :value="source.code">
              {{ source.display_name }}
            </option>
          </select>
        </label>
        <label>
          <span>{{ t('Connection') }}</span>
          <select v-model="state.replayConnectionId.value" class="hermes-select-control">
            <option value="">{{ t('All source connections') }}</option>
            <option v-for="connection in replayScopedConnections" :key="connection.id" :value="connection.id">
              {{ connection.display_name }} / {{ connection.status }}
            </option>
          </select>
        </label>
        <label>
          <span>{{ t('Event Pattern') }}</span>
          <input v-model="state.replayEventPattern.value" class="hermes-input-control" type="text" :placeholder="t('signal.raw.telegram.*')" />
        </label>
        <label>
          <span>{{ t('Selector Mode') }}</span>
          <select v-model="state.replaySelectorMode.value" class="hermes-select-control">
            <option value="all">{{ t('Whole pattern') }}</option>
            <option value="position">{{ t('Event log position') }}</option>
            <option value="time">{{ t('Occurred time') }}</option>
          </select>
        </label>
        <div v-if="state.replaySelectorMode.value === 'position'" class="replay-selector-grid">
          <label><span>{{ t('From Position') }}</span><input v-model="state.replayFromPosition.value" class="hermes-input-control" type="text" inputmode="numeric" placeholder="10" /></label>
          <label><span>{{ t('To Position') }}</span><input v-model="state.replayToPosition.value" class="hermes-input-control" type="text" inputmode="numeric" placeholder="20" /></label>
        </div>
        <div v-else-if="state.replaySelectorMode.value === 'time'" class="replay-selector-grid">
          <label><span>{{ t('From Time') }}</span><input v-model="state.replayFromTime.value" class="hermes-input-control" type="text" placeholder="2026-06-23T00:00:00Z" /></label>
          <label><span>{{ t('To Time') }}</span><input v-model="state.replayToTime.value" class="hermes-input-control" type="text" placeholder="2026-06-23T01:00:00Z" /></label>
        </div>
        <label>
          <span>{{ t('Target Consumer') }}</span>
          <select v-model="state.replayTargetConsumer.value" class="hermes-select-control">
            <option value="">{{ t('All downstream consumers') }}</option>
            <option v-for="runtimeKind in replayTargetConsumers" :key="runtimeKind" :value="runtimeKind">{{ runtimeKind }}</option>
          </select>
        </label>
        <label>
          <span>{{ t('Target Projection') }}</span>
          <select v-model="state.replayTargetProjection.value" class="hermes-select-control">
            <option value="">{{ t('No projection rebuild') }}</option>
            <option value="communication_messages">{{ t('Communications accepted-signal projection') }}</option>
            <option value="person_derived_evidence">{{ t('Person derived-evidence projection') }}</option>
            <option value="project_link_review_effects">{{ t('Project link-review effects projection') }}</option>
            <option value="timeline_event_log">{{ t('Timeline event-log projection') }}</option>
          </select>
        </label>
        <div class="signal-callout">
          <strong>{{ t('Replay target') }}</strong>
          <span>{{
            state.replayTargetProjection.value
              ? t('Selected projection rebuild runs against the matching event-log slice and rewinds the projection cursor before advancing it again.')
              : state.replayTargetConsumer.value
                ? t('Selected consumer cursor will be rewound to the requested signal range. Already processed events still respect consumer idempotency.')
                : t('Without a target consumer, replay re-enters the normal accepted-signal flow from paused-buffer or event-log selectors.')
          }}</span>
        </div>
        <button type="submit" class="hermes-btn" :disabled="state.isCreatingReplayRequest.value">
          <Icon icon="tabler:player-track-next" />
          {{ state.isCreatingReplayRequest.value ? t('Queueing') : t('Request Replay') }}
        </button>
      </form>

      <div v-if="replayRequests.length === 0" class="empty-panel fill">{{ t('No replay requests yet.') }}</div>
      <div v-else class="signal-table">
        <div v-for="request in replayRequests" :key="request.id" class="signal-table-row">
          <div class="signal-table-main">
            <Icon :icon="sourceIconForCode(sources, request.source_code ?? 'fixture')" />
            <span>
              <strong>{{ request.event_pattern ?? (request.connection_id ? connectionLabel(t, connections, request.connection_id) : null) ?? request.source_code ?? t('Global replay') }}</strong>
              <em>{{ request.requested_by }}</em>
            </span>
          </div>
          <b class="signal-pill" :data-tone="statusTone(request.status)">{{ request.status }}</b>
          <small>{{ `${request.replayed_count} ${t('events')} / ${state.describeSignalHubReplayRequest(request)}` }}</small>
        </div>
      </div>
    </div>
  </div>

  <div v-else class="signal-placeholder">
    <Icon :icon="state.tabs.find((tab) => tab.id === state.activeTab.value)?.icon ?? 'tabler:activity'" />
    <strong>{{ t(state.tabs.find((tab) => tab.id === state.activeTab.value)?.label ?? 'Signal Hub') }}</strong>
    <span>{{ t('No records in this section yet.') }}</span>
  </div>
</template>
