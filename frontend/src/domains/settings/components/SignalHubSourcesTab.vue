<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import { useI18n } from '../../../platform/i18n'
import type { SignalHubSettingsController } from './useSignalHubSettingsController'
import {
  capabilityLabel,
  capabilityLabels,
  capabilityTone,
  sourceControlState,
  sourceIcon,
  sourceStateTone
} from './signalHubSettingsPresentation'

const props = defineProps<{ state: SignalHubSettingsController }>()
const { t } = useI18n()

const sources = computed(() => props.state.sources.value)
const policies = computed(() => props.state.policies.value)
const categories = computed(() => props.state.categories.value)
const filteredSources = computed(() => props.state.filteredSources.value)
const selectedSource = computed(() => props.state.selectedSource.value)
const selectedSourceCapabilities = computed(() => props.state.selectedSourceCapabilities.value)
const fixtureSources = computed(() => props.state.fixtureSources.value)
</script>

<template>
  <div class="signal-sources-layout">
    <div class="source-catalog">
      <div class="source-toolbar">
        <input
          v-model="state.sourceSearch"
          class="hermes-input-control"
          type="search"
          :placeholder="t('Search sources')"
        />
        <select v-model="state.sourceCategory" class="hermes-select-control">
          <option v-for="category in categories" :key="category" :value="category">
            {{ category === 'all' ? t('All') : category }}
          </option>
        </select>
      </div>

      <div v-if="state.isLoading.value && sources.length === 0" class="empty-panel fill">
        {{ t('Loading sources...') }}
      </div>
      <div v-else-if="filteredSources.length === 0" class="empty-panel fill">
        {{ t('No matching sources.') }}
      </div>
      <div v-else class="source-list">
        <button
          v-for="source in filteredSources"
          :key="source.code"
          type="button"
          class="source-row"
          :class="{ selected: selectedSource?.code === source.code }"
          @click="state.selectedSourceCode.value = source.code"
        >
          <Icon :icon="sourceIcon(source)" />
          <span>
            <strong>{{ source.display_name }}</strong>
            <em>{{ source.code }}</em>
          </span>
          <b class="signal-pill" :data-tone="sourceStateTone(sourceControlState(policies, source))">
            {{ sourceControlState(policies, source) }}
          </b>
        </button>
      </div>
    </div>

    <aside v-if="selectedSource" class="source-inspector">
      <header>
        <Icon :icon="sourceIcon(selectedSource)" />
        <div>
          <h3>{{ selectedSource.display_name }}</h3>
          <span>{{ selectedSource.category }} / {{ selectedSource.source_kind }}</span>
        </div>
      </header>
      <dl>
        <div>
          <dt>{{ t('Code') }}</dt>
          <dd>{{ selectedSource.code }}</dd>
        </div>
        <div>
          <dt>{{ t('Schema') }}</dt>
          <dd>v{{ selectedSource.capability_schema_version }}</dd>
        </div>
        <div>
          <dt>{{ t('State') }}</dt>
          <dd>{{ sourceControlState(policies, selectedSource) }}</dd>
        </div>
        <div>
          <dt>{{ t('Updated') }}</dt>
          <dd>{{ selectedSource.updated_at }}</dd>
        </div>
      </dl>
      <div class="capability-grid">
        <span v-for="capability in capabilityLabels(selectedSource)" :key="capability">
          {{ t(capability) }}
        </span>
      </div>
      <div v-if="selectedSourceCapabilities.length > 0" class="signal-table capability-table">
        <div
          v-for="capability in selectedSourceCapabilities"
          :key="capability.id"
          class="signal-table-row"
        >
          <div class="signal-table-main">
            <Icon icon="tabler:bolt" />
            <span>
              <strong>{{ capabilityLabel(capability) }}</strong>
              <em>{{ capability.reason ?? t('No capability note') }}</em>
            </span>
          </div>
          <b class="signal-pill" :data-tone="capabilityTone(capability.state)">
            {{ capability.state }}
          </b>
          <div class="runtime-actions">
            <small>{{
              capability.requires_confirmation ? t('Confirmation required') : t('No confirmation')
            }}</small>
          </div>
        </div>
      </div>
      <div class="runtime-actions">
        <button
          type="button"
          class="hermes-btn hermes-btn--outline hermes-btn--compact"
          :disabled="state.isUpdatingSignalControls.value"
          @click="state.handleEnableSource(selectedSource.code)"
        >
          <Icon icon="tabler:player-play" />
          {{ t('Enable Source') }}
        </button>
        <button
          type="button"
          class="hermes-btn hermes-btn--outline hermes-btn--compact"
          :disabled="state.isUpdatingSignalControls.value"
          @click="state.handleDisableSource(selectedSource.code)"
        >
          <Icon icon="tabler:player-stop" />
          {{ t('Disable Source') }}
        </button>
      </div>
      <form
        v-if="selectedSource.code === 'fixture'"
        class="fixture-emit-form"
        @submit.prevent="state.handleEmitFixtureSignal"
      >
        <label>
          <span>{{ t('Fixture Signal') }}</span>
          <select v-model="state.fixtureSignalId.value" class="hermes-select-control">
            <option
              v-for="fixture in fixtureSources"
              :key="fixture.fixture_id"
              :value="fixture.fixture_id"
            >
              {{ `${fixture.fixture_id} / ${fixture.summary}` }}
            </option>
          </select>
        </label>
        <button type="submit" class="hermes-btn" :disabled="state.isEmittingFixture.value">
          <Icon icon="tabler:test-pipe" />
          {{ state.isEmittingFixture.value ? t('Emitting') : t('Emit Fixture') }}
        </button>
        <small v-if="fixtureSources.length > 0">
          {{
            fixtureSources.find((fixture) => fixture.fixture_id === state.fixtureSignalId.value)?.event_type ??
            t('No fixture catalog entry')
          }}
        </small>
        <small v-if="state.emitFixture.data.value">
          {{ `${state.emitFixture.data.value.event_type} / ${state.emitFixture.data.value.raw_event_id}` }}
        </small>
      </form>
    </aside>
  </div>
</template>
