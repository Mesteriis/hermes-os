<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { useSignalHubSettingsSurface } from '../queries/useSignalHubSettingsSurface'
import {
  healthTone,
  signalHubViewPresentation,
  sourceIcon,
  sourceStateTone
} from './signalHubSettingsPresentation'
import {
  signalControlAvailability,
  signalTargetIcon,
  signalTargetTone
} from './signalHubRoutePresentation'
import { useSignalHubSettingsPanelController } from '../queries/useSignalHubSettingsPanelController'

type SignalHubSettingsSurface = ReturnType<typeof useSignalHubSettingsSurface>

const props = defineProps<{
  surface: SignalHubSettingsSurface
}>()

const { t } = useI18n()
const {
  activeSignalViewPresentation,
  handleSelectGraphSource,
  handleSelectInventorySource,
  handleSelectSignalView,
  handlePauseSourceSignals,
  handleResumeSourceSignals,
  handleMuteSourceSignals,
  handleUnmuteSourceSignals,
  handleDisableSource,
  handleEnableSource,
} = useSignalHubSettingsPanelController({
  surface: props.surface,
})
</script>

<template>
  <section class="settings-section settings-signal-section">
    <header class="settings-section-toolbar">
      <div>
        <h3>{{ t('Signal Hub') }}</h3>
        <p>{{ t('Signal source controls stay under Settings; provider account setup stays in Accounts.') }}</p>
      </div>
    </header>

    <div v-if="surface.isLoading.value" class="settings-empty-state">
      <Icon icon="tabler:loader-2" />
      <strong>{{ t('Loading Signal Hub') }}</strong>
    </div>

    <template v-else>
      <section class="settings-signal-summary" :aria-label="t('Signal Hub summary')">
        <article class="settings-signal-summary-tile">
          <span>{{ t('Sources') }}</span>
          <strong>{{ surface.signalInventoryRows.value.length }}</strong>
        </article>
        <article class="settings-signal-summary-tile">
          <span>{{ t('Running') }}</span>
          <strong>{{ surface.enabledCount.value }}</strong>
        </article>
        <article class="settings-signal-summary-tile">
          <span>{{ t('Connected') }}</span>
          <strong>{{ surface.connectedCount.value }}</strong>
        </article>
        <article class="settings-signal-summary-tile">
          <span>{{ t('Attention') }}</span>
          <strong>{{ surface.unhealthyCount.value + surface.replayPendingCount.value }}</strong>
        </article>
      </section>

      <section
        class="settings-signal-panel"
        :aria-label="t(activeSignalViewPresentation.ariaLabel)"
      >
        <header class="settings-signal-panel__header">
          <div>
            <span>{{ t(activeSignalViewPresentation.eyebrow) }}</span>
            <strong>{{ t(activeSignalViewPresentation.title) }}</strong>
          </div>
          <small>{{ t(activeSignalViewPresentation.description) }}</small>
          <nav class="settings-signal-view-tabs" :aria-label="t('Signal Hub views')">
            <button
              v-for="view in surface.signalViewTabs.value"
              :key="view.id"
              type="button"
              class="settings-signal-view-tab"
              :class="{ active: surface.activeSignalView.value === view.id }"
              :aria-pressed="surface.activeSignalView.value === view.id"
              @click="handleSelectSignalView(view.id)"
            >
              <Icon :icon="signalHubViewPresentation(view.id).icon" />
              <span>{{ t(view.label) }}</span>
              <strong>{{ view.count }}</strong>
            </button>
          </nav>
        </header>

        <nav
          v-if="surface.activeSignalView.value === 'graph'"
          class="settings-signal-category-tabs"
          :aria-label="t('Signal categories')"
        >
          <button
            v-for="category in surface.graphSourceTabs.value"
            :key="category.id"
            type="button"
            class="settings-signal-category-tab"
            :class="{ active: surface.selectedGraphSourceCode.value === category.id }"
            :aria-pressed="surface.selectedGraphSourceCode.value === category.id"
            @click="handleSelectGraphSource(category.id)"
          >
            <span>{{ t(category.label) }}</span>
            <strong>{{ category.count }}</strong>
          </button>
        </nav>

        <nav v-else class="settings-signal-category-tabs" :aria-label="t('Signal inventory categories')">
          <button
            v-for="category in surface.inventorySourceTabs.value"
            :key="category.id"
            type="button"
            class="settings-signal-category-tab"
            :class="{ active: surface.selectedInventorySourceCode.value === category.id }"
            :aria-pressed="surface.selectedInventorySourceCode.value === category.id"
            @click="handleSelectInventorySource(category.id)"
          >
            <span>{{ t(category.label) }}</span>
            <strong>{{ category.count }}</strong>
          </button>
        </nav>

        <div v-if="surface.activeSignalView.value === 'graph'" class="settings-signal-graph">
          <article
            v-for="route in surface.filteredSignalConsumerGraph.value"
            :key="route.source.code"
            class="settings-signal-route"
          >
            <div class="settings-signal-node settings-signal-node--source">
              <i class="settings-provider-icon" aria-hidden="true">
                <Icon :icon="sourceIcon(route.source)" />
              </i>
              <span>
                <strong>{{ route.source.display_name }}</strong>
                <small>{{ route.source.code }}</small>
              </span>
              <em :class="`is-${sourceStateTone(route.state)}`">{{ t(route.state) }}</em>
            </div>

            <div class="settings-signal-node settings-signal-node--patterns">
              <code>{{ route.raw_pattern }}</code>
              <code>{{ route.accepted_pattern }}</code>
            </div>

            <div class="settings-signal-node settings-signal-node--targets">
              <span
                v-for="target in route.targets"
                :key="`${target.kind}:${target.id}`"
                class="settings-signal-chip"
                :class="[`is-${target.kind}`, `tone-${signalTargetTone(target)}`]"
              >
                <Icon :icon="signalTargetIcon(target.kind)" />
                {{ target.label }}
              </span>
            </div>
          </article>
        </div>

        <div v-else class="settings-signal-table-scroll">
          <table class="settings-signal-table">
            <thead>
              <tr>
                <th scope="col">{{ t('Signal') }}</th>
                <th scope="col">{{ t('State') }}</th>
                <th scope="col">{{ t('Consumed by') }}</th>
                <th scope="col">{{ t('Health') }}</th>
                <th scope="col">{{ t('Controls') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="row in surface.filteredSignalInventoryRows.value"
                :key="row.source.code"
              >
                <td>
                  <div class="settings-signal-identity">
                    <i class="settings-provider-icon" aria-hidden="true">
                      <Icon :icon="sourceIcon(row.source)" />
                    </i>
                    <span>
                      <strong>{{ row.source.display_name }}</strong>
                      <small>{{ row.raw_pattern }}</small>
                    </span>
                  </div>
                </td>
                <td>
                  <span class="settings-signal-state" :class="`is-${sourceStateTone(row.state)}`">
                    {{ t(row.state) }}
                  </span>
                  <small>{{ row.active_policies.length }} {{ t('policies') }}</small>
                </td>
                <td>
                  <div class="settings-signal-chip-list">
                    <span
                      v-for="target in row.targets"
                      :key="`${target.kind}:${target.id}`"
                      class="settings-signal-chip"
                      :class="[`is-${target.kind}`, `tone-${signalTargetTone(target)}`]"
                    >
                      {{ target.label }}
                    </span>
                  </div>
                </td>
                <td>
                  <div class="settings-signal-health">
                    <span
                      class="settings-signal-state"
                      :class="`is-${healthTone(row.health?.level ?? 'unknown')}`"
                    >
                      {{ t(row.health?.level ?? 'unknown') }}
                    </span>
                    <small>{{ row.health?.summary ?? t('No health history') }}</small>
                    <small>
                      {{ row.connection_count }} {{ t('connections') }} ·
                      {{ row.runtime_states.length }} {{ t('runtimes') }} ·
                      {{ row.capabilities.length }} {{ t('capabilities') }}
                    </small>
                  </div>
                </td>
                <td>
                  <div class="settings-signal-controls">
                    <button
                      type="button"
                      class="icon-button"
                      :title="t('Pause')"
                      :aria-label="`${t('Pause')} ${row.source.display_name}`"
                      :disabled="signalControlAvailability(row, surface.isUpdatingSignalControls.value).pauseDisabled"
                      @click="handlePauseSourceSignals(row.source.code)"
                    >
                      <Icon icon="tabler:player-pause" />
                    </button>
                    <button
                      type="button"
                      class="icon-button"
                      :title="t('Resume')"
                      :aria-label="`${t('Resume')} ${row.source.display_name}`"
                      :disabled="signalControlAvailability(row, surface.isUpdatingSignalControls.value).resumeDisabled"
                      @click="handleResumeSourceSignals(row.source.code)"
                    >
                      <Icon icon="tabler:player-play" />
                    </button>
                    <button
                      type="button"
                      class="icon-button"
                      :title="t('Mute')"
                      :aria-label="`${t('Mute')} ${row.source.display_name}`"
                      :disabled="signalControlAvailability(row, surface.isUpdatingSignalControls.value).muteDisabled"
                      @click="handleMuteSourceSignals(row.source.code)"
                    >
                      <Icon icon="tabler:volume-off" />
                    </button>
                    <button
                      type="button"
                      class="icon-button"
                      :title="t('Unmute')"
                      :aria-label="`${t('Unmute')} ${row.source.display_name}`"
                      :disabled="signalControlAvailability(row, surface.isUpdatingSignalControls.value).unmuteDisabled"
                      @click="handleUnmuteSourceSignals(row.source.code)"
                    >
                      <Icon icon="tabler:volume" />
                    </button>
                    <button
                      type="button"
                      class="icon-button danger-icon-button"
                      :title="t('Disable')"
                      :aria-label="`${t('Disable')} ${row.source.display_name}`"
                      :disabled="signalControlAvailability(row, surface.isUpdatingSignalControls.value).disableDisabled"
                      @click="handleDisableSource(row.source.code)"
                    >
                      <Icon icon="tabler:circle-off" />
                    </button>
                    <button
                      type="button"
                      class="icon-button"
                      :title="t('Enable')"
                      :aria-label="`${t('Enable')} ${row.source.display_name}`"
                      :disabled="signalControlAvailability(row, surface.isUpdatingSignalControls.value).enableDisabled"
                      @click="handleEnableSource(row.source.code)"
                    >
                      <Icon icon="tabler:power" />
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </template>
  </section>
</template>
