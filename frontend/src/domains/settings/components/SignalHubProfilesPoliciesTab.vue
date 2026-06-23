<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import { useI18n } from '../../../platform/i18n'
import type { SignalHubSettingsController } from './useSignalHubSettingsController'
import { policyTargetLabel, profilePolicyLabel } from './signalHubSettingsPresentation'

const props = defineProps<{ state: SignalHubSettingsController }>()
const { t } = useI18n()

const sources = computed(() => props.state.sources.value)
const connections = computed(() => props.state.connections.value)
const profiles = computed(() => props.state.profiles.value)
const policies = computed(() => props.state.policies.value)
const selectedProfile = computed(() => props.state.selectedProfile.value)
const connectionCapableSources = computed(() => props.state.connectionCapableSources.value)
const profileScopeConnections = computed(() => props.state.profileScopeConnections.value)
const policyScopeConnections = computed(() => props.state.policyScopeConnections.value)
</script>

<template>
  <template v-if="state.activeTab.value === 'profiles'">
    <div class="policy-layout">
      <form class="policy-form" @submit.prevent="state.handleSaveProfile">
        <label>
          <span>{{ t('Profile Code') }}</span>
          <input
            v-model="state.profileCodeInput.value"
            class="hermes-input-control"
            type="text"
            :placeholder="t('quiet_hours')"
            :disabled="Boolean(selectedProfile)"
          />
        </label>
        <label>
          <span>{{ t('Display Name') }}</span>
          <input
            v-model="state.profileDisplayNameInput.value"
            class="hermes-input-control"
            type="text"
            :placeholder="t('Quiet Hours')"
            :disabled="selectedProfile?.is_system"
          />
        </label>
        <label>
          <span>{{ t('Description') }}</span>
          <input
            v-model="state.profileDescriptionInput.value"
            class="hermes-input-control"
            type="text"
            :placeholder="t('What this profile changes')"
            :disabled="selectedProfile?.is_system"
          />
        </label>
        <div class="replay-selector-grid">
          <label>
            <span>{{ t('Scope') }}</span>
            <select v-model="state.profilePolicyScope.value" class="hermes-select-control" :disabled="selectedProfile?.is_system">
              <option value="event_pattern">{{ t('Event Pattern') }}</option>
              <option value="source">{{ t('Source') }}</option>
              <option value="connection">{{ t('Connection') }}</option>
              <option value="global">{{ t('Global') }}</option>
            </select>
          </label>
          <label>
            <span>{{ t('Mode') }}</span>
            <select v-model="state.profilePolicyMode.value" class="hermes-select-control" :disabled="selectedProfile?.is_system">
              <option value="paused">{{ t('Pause') }}</option>
              <option value="muted">{{ t('Mute') }}</option>
              <option value="disabled">{{ t('Disable') }}</option>
              <option value="enabled">{{ t('Allow') }}</option>
            </select>
          </label>
        </div>
        <label v-if="state.profilePolicyScope.value === 'source' || state.profilePolicyScope.value === 'connection'">
          <span>{{ t('Source') }}</span>
          <select v-model="state.profilePolicySourceCode.value" class="hermes-select-control" :disabled="selectedProfile?.is_system">
            <option
              v-for="source in state.profilePolicyScope.value === 'connection' ? connectionCapableSources : sources"
              :key="source.code"
              :value="source.code"
            >
              {{ source.display_name }}
            </option>
          </select>
        </label>
        <label v-if="state.profilePolicyScope.value === 'connection'">
          <span>{{ t('Connection') }}</span>
          <select v-model="state.profilePolicyConnectionId.value" class="hermes-select-control" :disabled="selectedProfile?.is_system">
            <option value="">{{ t('Select connection') }}</option>
            <option v-for="connection in profileScopeConnections" :key="connection.id" :value="connection.id">
              {{ connection.display_name }} / {{ connection.status }}
            </option>
          </select>
        </label>
        <label v-if="state.profilePolicyScope.value === 'event_pattern'">
          <span>{{ t('Pattern') }}</span>
          <input
            v-model="state.profilePolicyEventPattern.value"
            class="hermes-input-control"
            type="text"
            placeholder="signal.raw.*"
            :disabled="selectedProfile?.is_system"
          />
        </label>
        <label>
          <span>{{ t('Reason') }}</span>
          <input
            v-model="state.profilePolicyReason.value"
            class="hermes-input-control"
            type="text"
            :placeholder="t('Profile policy reason')"
            :disabled="selectedProfile?.is_system"
          />
        </label>
        <div class="runtime-actions">
          <button type="button" class="hermes-btn hermes-btn--outline hermes-btn--compact" :disabled="selectedProfile?.is_system" @click="state.addDraftProfilePolicy">
            <Icon icon="tabler:list-plus" />
            {{ t('Add Policy') }}
          </button>
        </div>
        <div class="signal-callout">
          <strong>{{ t('Profile Policies') }}</strong>
          <span v-if="state.profileDraftPolicies.value.length === 0">{{ t('No profile policies yet.') }}</span>
          <template v-else>
            <div
              v-for="(policy, index) in state.profileDraftPolicies.value"
              :key="`${policy.scope}:${policy.mode}:${index}`"
              class="policy-row"
            >
              <Icon icon="tabler:shield-cog" />
              <span>
                <strong>{{ profilePolicyLabel(t, connections, policy) }}</strong>
                <em>{{ policy.reason }}</em>
              </span>
              <button
                v-if="!selectedProfile?.is_system"
                type="button"
                class="hermes-btn hermes-btn--outline hermes-btn--compact"
                @click="state.removeDraftProfilePolicy(index)"
              >
                {{ t('Remove') }}
              </button>
            </div>
          </template>
        </div>
        <div class="runtime-actions">
          <button type="submit" class="hermes-btn" :disabled="state.isSavingProfile.value || selectedProfile?.is_system">
            <Icon icon="tabler:device-floppy" />
            {{
              selectedProfile
                ? state.isSavingProfile.value ? t('Saving') : t('Update Profile')
                : state.isSavingProfile.value ? t('Creating') : t('Create Profile')
            }}
          </button>
          <button type="button" class="hermes-btn hermes-btn--outline" @click="state.resetProfileEditor">
            {{ t('Reset') }}
          </button>
          <button
            v-if="selectedProfile && !selectedProfile.is_system"
            type="button"
            class="hermes-btn hermes-btn--outline"
            :disabled="state.isRemovingProfile.value"
            @click="state.handleRemoveProfile(selectedProfile.code)"
          >
            {{ state.isRemovingProfile.value ? t('Removing') : t('Delete Profile') }}
          </button>
        </div>
      </form>

      <div v-if="profiles.length === 0" class="empty-panel fill">{{ t('No profiles yet.') }}</div>
      <div v-else class="signal-table">
        <button
          v-for="profile in profiles"
          :key="profile.id"
          type="button"
          class="signal-table-row signal-runtime-row"
          :class="{ selected: selectedProfile?.code === profile.code }"
          @click="state.handleSelectProfile(profile)"
        >
          <div class="signal-table-main">
            <Icon icon="tabler:layout-dashboard" />
            <span>
              <strong>{{ profile.display_name }}</strong>
              <em>{{ profile.code }} / {{ profile.policy_count }} {{ t('policies') }}</em>
              <small class="signal-detail-text">{{ profile.description }}</small>
            </span>
          </div>
          <b class="signal-pill" :data-tone="profile.is_active ? 'good' : 'neutral'">
            {{ profile.is_active ? t('Active') : profile.is_system ? t('System') : t('Custom') }}
          </b>
          <div class="runtime-actions">
            <small class="profile-description">
              {{ profile.source_policies.slice(0, 2).map((policy) => profilePolicyLabel(t, connections, policy)).join(' • ') || t('No profile policies') }}
            </small>
            <button type="button" class="hermes-btn hermes-btn--outline hermes-btn--compact" :disabled="state.isApplyingProfile.value || profile.is_active" @click.stop="state.handleApplyProfile(profile.code)">
              {{ profile.is_active ? t('Applied') : t('Apply') }}
            </button>
          </div>
        </button>
      </div>
    </div>
  </template>

  <div v-else class="policy-layout">
    <form class="policy-form" @submit.prevent="state.handleCreatePolicy">
      <label>
        <span>{{ t('Scope') }}</span>
        <select v-model="state.policyScope.value" class="hermes-select-control">
          <option value="event_pattern">{{ t('Event Pattern') }}</option>
          <option value="source">{{ t('Source') }}</option>
          <option value="connection">{{ t('Connection') }}</option>
          <option value="global">{{ t('Global') }}</option>
        </select>
      </label>
      <label>
        <span>{{ t('Mode') }}</span>
        <select v-model="state.policyMode.value" class="hermes-select-control">
          <option value="paused">{{ t('Pause') }}</option>
          <option value="muted">{{ t('Mute') }}</option>
          <option value="disabled">{{ t('Disable') }}</option>
          <option value="enabled">{{ t('Allow') }}</option>
        </select>
      </label>
      <label v-if="state.policyScope.value === 'source'">
        <span>{{ t('Source') }}</span>
        <select v-model="state.policySourceCode.value" class="hermes-select-control">
          <option v-for="source in sources" :key="source.code" :value="source.code">{{ source.display_name }}</option>
        </select>
      </label>
      <label v-if="state.policyScope.value === 'connection'">
        <span>{{ t('Source') }}</span>
        <select v-model="state.policySourceCode.value" class="hermes-select-control">
          <option v-for="source in connectionCapableSources" :key="source.code" :value="source.code">{{ source.display_name }}</option>
        </select>
      </label>
      <label v-if="state.policyScope.value === 'connection'">
        <span>{{ t('Connection') }}</span>
        <select v-model="state.policyConnectionId.value" class="hermes-select-control">
          <option value="">{{ t('Select connection') }}</option>
          <option v-for="connection in policyScopeConnections" :key="connection.id" :value="connection.id">
            {{ connection.display_name }} / {{ connection.status }}
          </option>
        </select>
      </label>
      <label v-if="state.policyScope.value === 'event_pattern'">
        <span>{{ t('Pattern') }}</span>
        <input v-model="state.policyEventPattern.value" class="hermes-input-control" type="text" placeholder="signal.raw.*" />
      </label>
      <label>
        <span>{{ t('Reason') }}</span>
        <input v-model="state.policyReason.value" class="hermes-input-control" type="text" :placeholder="t('Policy reason')" />
      </label>
      <button type="submit" class="hermes-btn" :disabled="state.isCreatingPolicy.value || state.isUpdatingSignalControls.value">
        <Icon icon="tabler:shield-plus" />
        {{ state.isCreatingPolicy.value || state.isUpdatingSignalControls.value ? t('Saving') : t('Create Policy') }}
      </button>
    </form>

    <div class="policy-list">
      <div v-if="policies.length === 0" class="empty-panel fill">{{ t('No active policies.') }}</div>
      <template v-else>
        <div
          v-for="policy in policies"
          :key="`${policy.scope}:${policy.mode}:${policy.source_code ?? policy.event_pattern ?? 'global'}`"
          class="policy-row"
        >
          <Icon icon="tabler:shield-cog" />
          <span>
            <strong>{{ policy.mode }}</strong>
            <em>{{ policyTargetLabel(t, connections, policy) }}</em>
          </span>
          <b>{{ policy.reason }}</b>
          <button
            v-if="policy.mode === 'paused' || policy.mode === 'muted' || (policy.mode === 'disabled' && policy.scope === 'source')"
            type="button"
            class="hermes-btn hermes-btn--outline hermes-btn--compact"
            :disabled="state.isUpdatingSignalControls.value"
            @click="state.handleClearPolicy(policy)"
          >
            {{ policy.mode === 'paused' ? t('Resume') : policy.mode === 'muted' ? t('Unmute') : t('Enable') }}
          </button>
        </div>
      </template>
    </div>
  </div>
</template>
