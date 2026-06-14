<script setup lang="ts">
import { computed } from 'vue'
import { useQueryClient } from '@tanstack/vue-query'
import { useI18n } from '../../../platform/i18n'
import {
  useSidebarStore,
  type SidebarItemId,
  type SidebarNavGroup,
  type SidebarSettings as SidebarSettingsValue
} from '../../../shared/stores/sidebar'
import { saveApplicationSetting } from '../api/settings'
import { settingsKeys } from '../queries/useSettingsQuery'
import { useSettingsStore } from '../stores/settings'
import {
  FRONTEND_SIDEBAR_SETTING_KEY,
  type ApplicationSettingValue
} from '../types/settings'
import SidebarGroupEditor from './sidebar/SidebarGroupEditor.vue'
import SidebarNavigationList from './sidebar/SidebarNavigationList.vue'
import SidebarSettingsSummary from './sidebar/SidebarSettingsSummary.vue'

const { t } = useI18n()
const sidebar = useSidebarStore()
const store = useSettingsStore()
const queryClient = useQueryClient()

const sidebarItemLabels = computed<Record<SidebarItemId, { label: string; icon: string }>>(() => {
  const labels = {} as Record<SidebarItemId, { label: string; icon: string }>
  const itemIds = new Set<SidebarItemId>([
    ...sidebar.effectiveSidebarSettings.hiddenItemIds,
    ...sidebar.effectiveSidebarSettings.groups.flatMap((group) => group.itemIds),
    ...sidebar.sidebarRootEntries.flatMap((entry) => entry.kind === 'item' ? [entry.item.itemId] : [])
  ])

  for (const itemId of itemIds) {
    const item = sidebar.sidebarConfigItem(itemId)
    if (item) labels[itemId] = { label: item.label, icon: item.icon }
  }

  return labels
})

const sidebarGroupOptions = computed(() =>
  sidebar.effectiveSidebarSettings.groups.map((group, index) => ({
    value: group.id,
    label: sidebarGroupLabel(group, index)
  }))
)

const sidebarRuleSummaries = computed(() => [
  { text: t('Default keeps the current sidebar order'), badge: t('Preset') },
  { text: t('Communications sources stay nested'), badge: t('Context') },
  { text: t('Hidden domains stay recoverable here'), badge: t('Safe') },
  { text: t('Settings store no message content'), badge: t('Privacy') }
])

function sidebarGroupLabel(group: SidebarNavGroup, index: number): string {
  return group.label || (group.id === 'communications' ? 'Communications' : `Group ${index + 1}`)
}

function sidebarRootIndexForGroup(groupId: string): number {
  const normalized = sidebar.sidebarGroupIdFromLabel(groupId)
  return sidebar.effectiveSidebarSettings.rootItemIds.indexOf(`group:${normalized}`)
}

function toApplicationSettingValue(settings: SidebarSettingsValue): ApplicationSettingValue {
  return {
    schemaVersion: settings.schemaVersion,
    rootItemIds: [...settings.rootItemIds],
    groups: settings.groups.map((group) => ({
      id: group.id,
      label: group.label,
      icon: group.icon,
      itemIds: [...group.itemIds],
      separatorBeforeItemIds: [...group.separatorBeforeItemIds]
    })),
    hiddenItemIds: [...settings.hiddenItemIds]
  }
}

async function handleSaveSidebar(): Promise<void> {
  store.isSidebarSettingsSaving = true
  store.sidebarError = ''
  try {
    await saveApplicationSetting(
      FRONTEND_SIDEBAR_SETTING_KEY,
      toApplicationSettingValue(sidebar.effectiveSidebarSettings)
    )
    sidebar.setSidebarSettings(sidebar.effectiveSidebarSettings)
    queryClient.invalidateQueries({ queryKey: settingsKeys.application() })
    store.setActionMessage(t('Sidebar saved'))
  } catch (err) {
    store.sidebarError = err instanceof Error ? err.message : t('Failed to save sidebar')
  } finally {
    store.isSidebarSettingsSaving = false
  }
}

</script>

<template>
  <div class="settings-layout sidebar-settings-layout">
    <section class="panel settings-list-panel settings-primary-pane sidebar-settings-panel">
      <header class="panel-title-row">
        <div>
          <h2>{{ t('Sidebar Navigation') }}</h2>
          <p>{{ t('Configure workspace groups, order and hidden domains.') }}</p>
        </div>
        <div class="sidebar-settings-actions">
          <button
            type="button"
            class="hermes-btn hermes-btn--outline"
            :disabled="!sidebar.sidebarDraft || store.isSidebarSettingsSaving"
            @click="sidebar.cancelSidebarSettingsEditing()"
          >
            {{ t('Cancel') }}
          </button>
          <button
            type="button"
            class="hermes-btn hermes-btn--outline"
            :disabled="store.isSidebarSettingsSaving"
            @click="sidebar.resetSidebarSettingsToDefault()"
          >
            {{ t('Default') }}
          </button>
          <button
            type="button"
            class="hermes-btn hermes-btn--primary"
            :disabled="!sidebar.sidebarDraft || store.isSidebarSettingsSaving"
            @click="handleSaveSidebar"
          >
            {{ store.isSidebarSettingsSaving ? t('Saving...') : t('Save') }}
          </button>
        </div>
      </header>

      <div v-if="store.sidebarError" class="inline-error">{{ store.sidebarError }}</div>

      <form class="sidebar-group-create" @submit.prevent="sidebar.addSidebarGroup()">
        <label>
          <span>{{ t('New group') }}</span>
          <input
            v-model="store.newSidebarGroupLabel"
            :placeholder="t('Focus, Library, Planning')"
            autocomplete="off"
          />
        </label>
        <button type="submit" class="hermes-btn hermes-btn--secondary">
          {{ t('Create Group') }}
        </button>
      </form>

      <div class="sidebar-config-list">
        <SidebarNavigationList
          :entries="sidebar.sidebarRootEntries"
          :hidden-item-ids="sidebar.sidebarHiddenNavItems"
          :root-item-count="sidebar.effectiveSidebarSettings.rootItemIds.length"
          :group-options="sidebarGroupOptions"
          :root-label="t('Root level')"
          :sidebar-root-label="t('Sidebar root')"
          :expandable-group-label="t('Expandable group')"
          :items-label="t('items')"
          :hidden-label="t('Hidden from sidebar')"
          :root-domain-label="t('Root domain')"
          :move-to-group-label="t('Move to group')"
          :show-label="t('Show')"
          :hide-label="t('Hide')"
          @move-group="sidebar.moveSidebarGroup"
          @remove-group="sidebar.removeSidebarGroup"
          @move-root-item="sidebar.moveSidebarRootItem"
          @move-item-to-group="sidebar.moveSidebarItemToGroup"
          @toggle-hidden="sidebar.toggleSidebarItemHidden"
        />

        <SidebarGroupEditor
          v-for="(group, groupIndex) in sidebar.effectiveSidebarSettings.groups"
          :key="group.id"
          :group="group"
          :group-index="groupIndex"
          :root-index="sidebarRootIndexForGroup(group.id)"
          :root-item-count="sidebar.effectiveSidebarSettings.rootItemIds.length"
          :item-labels="sidebarItemLabels"
          :hidden-item-ids="sidebar.sidebarHiddenNavItems"
          :group-options="sidebarGroupOptions"
          :group-label-text="t('Group label')"
          :default-placeholder="t('Primary')"
          :group-placeholder="t('Group {n}').replace('{n}', String(groupIndex + 1))"
          :visible-domain-label="t('Visible domain')"
          :hidden-label="t('Hidden from sidebar')"
          :no-items-label="t('No items in this group.')"
          :move-to-group-label="t('Move to group')"
          :divider-label="t('Divider')"
          :show-label="t('Show')"
          :hide-label="t('Hide')"
          @rename="sidebar.updateSidebarGroupLabel"
          @move-group="sidebar.moveSidebarGroup"
          @remove-group="sidebar.removeSidebarGroup"
          @move-item-to-group="sidebar.moveSidebarItemToGroup"
          @move-item="sidebar.moveSidebarItem"
          @toggle-divider="sidebar.toggleSidebarGroupSeparator"
          @toggle-hidden="sidebar.toggleSidebarItemHidden"
        />
      </div>
    </section>

    <SidebarSettingsSummary
      :entries="sidebar.sidebarRootEntries"
      :hidden-item-ids="sidebar.sidebarHiddenNavItems"
      :item-labels="sidebarItemLabels"
      :preview-label="t('Preview')"
      :hidden-label="t('Hidden')"
      :rules-label="t('Rules')"
      :root-domain-label="t('Root domain')"
      :empty-group-label="t('Empty group')"
      :no-hidden-label="t('No domains are hidden.')"
      :show-label="t('Show')"
      :rules="sidebarRuleSummaries"
      @toggle-hidden="sidebar.toggleSidebarItemHidden"
    />
  </div>
</template>

<style scoped>
.sidebar-settings-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(260px, 310px);
  gap: var(--hh-layout-gap);
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.sidebar-settings-panel {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}

.sidebar-settings-actions,
.sidebar-group-create {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.sidebar-group-create {
  align-items: center;
  padding: 12px;
  border-top: 1px solid var(--hh-border);
}

.sidebar-group-create label {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  font-size: 12px;
  color: var(--hh-text-secondary);
}

.sidebar-group-create input {
  flex: 1;
  min-width: 0;
  height: 34px;
  padding: 0 10px;
  background: var(--hh-surface-deep);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-text-primary);
  font-size: 12px;
  outline: none;
}

.sidebar-group-create input:focus-visible {
  box-shadow: 0 0 0 2px var(--hh-focus-ring);
  border-color: var(--hh-accent);
}

.sidebar-config-list {
  flex: 1 1 auto;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
  padding: 8px;
}

.inline-error {
  padding: 8px 12px;
  background: color-mix(in srgb, var(--hh-status-danger) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--hh-status-danger) 30%, transparent);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-status-danger);
  font-size: 12px;
  margin: 8px;
}
</style>
