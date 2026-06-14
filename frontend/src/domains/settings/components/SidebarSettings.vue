<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import { useSidebarStore, type SidebarItemId, type SidebarNavGroup, type ResolvedSidebarItem, type ResolvedSidebarRootEntry } from '../../../shared/stores/sidebar'
import { useSettingsStore } from '../stores/settings'
import { saveApplicationSetting } from '../api/settings'
import { FRONTEND_SIDEBAR_SETTING_KEY } from '../types/settings'
import { useQueryClient } from '@tanstack/vue-query'
import { settingsKeys } from '../queries/useSettingsQuery'

const { t } = useI18n()
const sidebar = useSidebarStore()
const store = useSettingsStore()
const queryClient = useQueryClient()

function sidebarGroupLabel(group: SidebarNavGroup, index: number): string {
  return group.label || (group.id === 'communications' ? 'Communications' : `Group ${index + 1}`)
}

function sidebarItemLabel(item: ResolvedSidebarItem): string {
  return item.label
}

function sidebarGroupHasSeparatorBefore(group: SidebarNavGroup, itemId: SidebarItemId): boolean {
  return group.itemIds.indexOf(itemId) > 0 && group.separatorBeforeItemIds.includes(itemId)
}

function sidebarRootIndexForGroup(groupId: string): number {
  return sidebar.effectiveSidebarSettings.rootItemIds.indexOf(`group:${sidebar.sidebarGroupIdFromLabel(groupId)}`)
}

function sidebarConfigItem(itemId: SidebarItemId): { id: SidebarItemId; label: string; icon: string } | null {
  return sidebar.sidebarConfigItem(itemId)
}

function sidebarGroupIdFromLabelFn(label: string): string {
  if (label.startsWith('group:')) {
    const groupId = label.slice('group:'.length)
    return sidebar.effectiveSidebarSettings.groups.find((g) => sidebar.sidebarGroupIdFromLabel(g.id) === groupId)?.id ?? ''
  }
  return ''
}

function sidebarMoveTargetOptions(includeRoot: boolean) {
  return [
    ...(includeRoot ? [{ value: 'root', label: t('Root level') }] : []),
    ...sidebar.effectiveSidebarSettings.groups.map((group, index) => ({
      value: group.id,
      label: sidebarGroupLabel(group, index)
    }))
  ]
}

async function handleSaveSidebar() {
  store.isSidebarSettingsSaving = true
  store.sidebarError = ''
  try {
    await saveApplicationSetting(FRONTEND_SIDEBAR_SETTING_KEY, sidebar.effectiveSidebarSettings as any)
    sidebar.setSidebarSettings(sidebar.effectiveSidebarSettings)
    queryClient.invalidateQueries({ queryKey: settingsKeys.application() })
    store.setActionMessage(t('Sidebar saved'))
  } catch (err) {
    store.sidebarError = err instanceof Error ? err.message : t('Failed to save sidebar')
  } finally {
    store.isSidebarSettingsSaving = false
  }
}

function handleMoveSidebarItemToGroup(itemId: SidebarItemId, targetGroupId: string) {
  sidebar.moveSidebarItemToGroup(itemId, targetGroupId)
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

      <!-- Create group form -->
      <form
        class="sidebar-group-create"
        @submit.prevent="sidebar.addSidebarGroup()"
      >
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
        <!-- Root level section -->
        <section class="sidebar-config-group">
          <header>
            <label>
              <span>{{ t('Root level') }}</span>
              <input :value="t('Sidebar root')" disabled autocomplete="off" />
            </label>
          </header>
          <div class="sidebar-config-items">
            <template v-for="(rootId, rootIndex) in sidebar.effectiveSidebarSettings.rootItemIds" :key="rootId">
              <!-- If it's a group rootId -->
              <template v-if="sidebarGroupIdFromLabelFn(rootId)">
                <div
                  v-for="group in sidebar.effectiveSidebarSettings.groups.filter(g => sidebarGroupIdFromLabelFn(rootId) === g.id)"
                  :key="group.id"
                  class="sidebar-config-item group-node"
                >
                  <div class="sidebar-config-item-main">
                    <span class="round-icon green">
                      {{ group.icon }}
                    </span>
                    <div>
                      <strong>{{ sidebarGroupLabel(group, rootIndex) }}</strong>
                      <small>{{ t('Expandable group') }} · {{ group.itemIds.length }} {{ t('items') }}</small>
                    </div>
                  </div>
                  <div class="sidebar-config-item-controls">
                    <button
                      type="button"
                      class="hermes-btn hermes-btn--icon"
                      :disabled="rootIndex === 0"
                      @click="sidebar.moveSidebarGroup(group.id, -1)"
                    >
                      ↑
                    </button>
                    <button
                      type="button"
                      class="hermes-btn hermes-btn--icon"
                      :disabled="rootIndex === sidebar.effectiveSidebarSettings.rootItemIds.length - 1"
                      @click="sidebar.moveSidebarGroup(group.id, 1)"
                    >
                      ↓
                    </button>
                    <button
                      type="button"
                      class="hermes-btn hermes-btn--icon hermes-btn--destructive"
                      :disabled="group.id === 'communications'"
                      @click="sidebar.removeSidebarGroup(group.id)"
                    >
                      ✕
                    </button>
                  </div>
                </div>
              </template>

              <!-- If it's a root item (not a group) -->
              <template v-else>
                <div
                  v-for="item in [sidebarConfigItem(rootId as SidebarItemId)].filter(Boolean)"
                  :key="item!.id"
                  class="sidebar-config-item"
                  :class="{ hidden: sidebar.effectiveSidebarSettings.hiddenItemIds.includes(item!.id) }"
                >
                  <div class="sidebar-config-item-main">
                    <span class="round-icon cyan">{{ item!.icon }}</span>
                    <div>
                      <strong>{{ item!.label }}</strong>
                      <small>{{ sidebar.effectiveSidebarSettings.hiddenItemIds.includes(item!.id) ? t('Hidden from sidebar') : t('Root domain') }}</small>
                    </div>
                  </div>
                  <div class="sidebar-config-item-controls">
                    <select
                      class="hermes-select-control"
                      value="root"
                      @change="(e) => handleMoveSidebarItemToGroup(item!.id, (e.target as HTMLSelectElement).value)"
                    >
                      <option value="root" disabled>{{ t('Move to group') }}</option>
                      <option
                        v-for="opt in sidebarMoveTargetOptions(true)"
                        :key="opt.value"
                        :value="opt.value"
                      >
                        {{ opt.label }}
                      </option>
                    </select>
                    <button
                      type="button"
                      class="hermes-btn hermes-btn--icon"
                      :disabled="rootIndex === 0"
                      @click="sidebar.moveSidebarRootItem(rootId, -1)"
                    >
                      ↑
                    </button>
                    <button
                      type="button"
                      class="hermes-btn hermes-btn--icon"
                      :disabled="rootIndex === sidebar.effectiveSidebarSettings.rootItemIds.length - 1"
                      @click="sidebar.moveSidebarRootItem(rootId, 1)"
                    >
                      ↓
                    </button>
                    <button
                      type="button"
                      class="hermes-btn hermes-btn--icon"
                      :class="{ active: !sidebar.effectiveSidebarSettings.hiddenItemIds.includes(item!.id) }"
                      @click="sidebar.toggleSidebarItemHidden(item!.id)"
                    >
                      {{ sidebar.effectiveSidebarSettings.hiddenItemIds.includes(item!.id) ? t('Show') : t('Hide') }}
                    </button>
                  </div>
                </div>
              </template>
            </template>
          </div>
        </section>

        <!-- Per-group sections -->
        <section
          v-for="(group, groupIndex) in sidebar.effectiveSidebarSettings.groups"
          :key="group.id"
          class="sidebar-config-group"
        >
          <header>
            <label>
              <span>{{ t('Group label') }}</span>
              <input
                :value="group.label"
                :placeholder="groupIndex === 0 ? t('Primary') : t('Group {n}').replace('{n}', String(groupIndex + 1))"
                autocomplete="off"
                @input="(e) => sidebar.updateSidebarGroupLabel(group.id, (e.target as HTMLInputElement).value)"
              />
            </label>
            <div class="sidebar-config-group-actions">
              <button
                type="button"
                class="hermes-btn hermes-btn--icon"
                :disabled="sidebarRootIndexForGroup(group.id) <= 0"
                @click="sidebar.moveSidebarGroup(group.id, -1)"
              >
                ↑
              </button>
              <button
                type="button"
                class="hermes-btn hermes-btn--icon"
                :disabled="sidebarRootIndexForGroup(group.id) === sidebar.effectiveSidebarSettings.rootItemIds.length - 1"
                @click="sidebar.moveSidebarGroup(group.id, 1)"
              >
                ↓
              </button>
              <button
                type="button"
                class="hermes-btn hermes-btn--icon hermes-btn--destructive"
                :disabled="group.id === 'communications'"
                @click="sidebar.removeSidebarGroup(group.id)"
              >
                ✕
              </button>
            </div>
          </header>
          <div class="sidebar-config-items">
            <div v-if="group.itemIds.length === 0" class="empty-panel">
              {{ t('No items in this group.') }}
            </div>
            <template v-else>
              <div
                v-for="(itemId, itemIndex) in group.itemIds"
                :key="itemId"
              >
                <div
                  v-for="item in [sidebarConfigItem(itemId)].filter(Boolean)"
                  :key="item!.id"
                  class="sidebar-config-item"
                  :class="{ hidden: sidebar.effectiveSidebarSettings.hiddenItemIds.includes(item!.id) }"
                >
                  <div class="sidebar-config-item-main">
                    <span class="round-icon cyan">{{ item!.icon }}</span>
                    <div>
                      <strong>{{ item!.label }}</strong>
                      <small>{{ sidebar.effectiveSidebarSettings.hiddenItemIds.includes(item!.id) ? t('Hidden from sidebar') : t('Visible domain') }}</small>
                    </div>
                  </div>
                  <div class="sidebar-config-item-controls">
                    <select
                      class="hermes-select-control"
                      :value="group.id"
                      @change="(e) => handleMoveSidebarItemToGroup(item!.id, (e.target as HTMLSelectElement).value)"
                    >
                      <option :value="group.id" disabled>{{ t('Move to group') }}</option>
                      <option
                        v-for="opt in sidebarMoveTargetOptions(false)"
                        :key="opt.value"
                        :value="opt.value"
                      >
                        {{ opt.label }}
                      </option>
                    </select>
                    <button
                      type="button"
                      class="hermes-btn hermes-btn--icon"
                      @click="sidebar.moveSidebarItem(item!.id, -1)"
                    >
                      ↑
                    </button>
                    <button
                      type="button"
                      class="hermes-btn hermes-btn--icon"
                      @click="sidebar.moveSidebarItem(item!.id, 1)"
                    >
                      ↓
                    </button>
                    <button
                      type="button"
                      class="hermes-btn hermes-btn--icon"
                      :class="{ active: sidebarGroupHasSeparatorBefore(group, item!.id) }"
                      :disabled="itemIndex === 0"
                      @click="sidebar.toggleSidebarGroupSeparator(group.id, item!.id)"
                    >
                      {{ t('Divider') }}
                    </button>
                    <button
                      type="button"
                      class="hermes-btn hermes-btn--icon"
                      :class="{ active: !sidebar.effectiveSidebarSettings.hiddenItemIds.includes(item!.id) }"
                      @click="sidebar.toggleSidebarItemHidden(item!.id)"
                    >
                      {{ sidebar.effectiveSidebarSettings.hiddenItemIds.includes(item!.id) ? t('Show') : t('Hide') }}
                    </button>
                  </div>
                </div>
              </div>
            </template>
          </div>
        </section>
      </div>
    </section>

    <aside class="settings-rail sidebar-settings-summary">
      <section class="panel info-card">
        <h2>{{ t('Preview') }}</h2>
        <ul class="sidebar-preview-list">
          <li v-for="(entry, entryIndex) in sidebar.sidebarRootEntries" :key="entryIndex">
            <template v-if="entry.kind === 'group'">
              <strong>{{ sidebarGroupLabel(entry.group, entryIndex) }}</strong>
              <span>{{ entry.group.items.map((i) => sidebarItemLabel(i)).join(', ') || t('Empty group') }}</span>
            </template>
            <template v-else>
              <strong>{{ sidebarItemLabel(entry.item) }}</strong>
              <span>{{ t('Root domain') }}</span>
            </template>
          </li>
        </ul>
      </section>
      <section class="panel info-card">
        <h2>{{ t('Hidden') }}</h2>
        <p v-if="sidebar.sidebarHiddenNavItems.length === 0">{{ t('No domains are hidden.') }}</p>
        <ul v-else class="detail-list">
          <li v-for="itemId in sidebar.sidebarHiddenNavItems" :key="itemId">
            <template v-for="item in [sidebarConfigItem(itemId)].filter(Boolean)" :key="item!.id">
              {{ item!.label }}
              <button type="button" class="hermes-btn hermes-btn--ghost" @click="sidebar.toggleSidebarItemHidden(item!.id)">
                {{ t('Show') }}
              </button>
            </template>
          </li>
        </ul>
      </section>
      <section class="panel info-card">
        <h2>{{ t('Rules') }}</h2>
        <ul class="detail-list">
          <li>{{ t('Default keeps the current sidebar order') }}<em>{{ t('Preset') }}</em></li>
          <li>{{ t('Communications sources stay nested') }}<em>{{ t('Context') }}</em></li>
          <li>{{ t('Hidden domains stay recoverable here') }}<em>{{ t('Safe') }}</em></li>
          <li>{{ t('Settings store no message content') }}<em>{{ t('Privacy') }}</em></li>
        </ul>
      </section>
    </aside>
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

.sidebar-settings-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.sidebar-group-create {
  display: flex;
  align-items: center;
  gap: 8px;
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

.sidebar-config-group {
  margin-bottom: 12px;
}

.sidebar-config-group > header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  border-bottom: 1px solid var(--hh-border);
}

.sidebar-config-group > header label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--hh-text-secondary);
}

.sidebar-config-group > header input {
  max-width: 180px;
  height: 28px;
  padding: 0 8px;
  background: var(--hh-surface-deep);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-text-primary);
  font-size: 12px;
  outline: none;
}

.sidebar-config-group > header input:focus-visible {
  box-shadow: 0 0 0 2px var(--hh-focus-ring);
  border-color: var(--hh-accent);
}

.sidebar-config-group-actions {
  display: flex;
  gap: 4px;
}

.sidebar-config-items {
  display: grid;
  gap: 4px;
  padding: 4px 0;
}

.sidebar-config-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  border-radius: var(--hh-radius-sm);
  transition: background 100ms ease;
}

.sidebar-config-item:hover {
  background: var(--hh-hover-bg);
}

.sidebar-config-item.hidden {
  opacity: 0.5;
}

.sidebar-config-item.group-node {
  border-left: 2px solid var(--hh-accent);
}

.sidebar-config-item-main {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.sidebar-config-item-main strong {
  display: block;
  font-size: 12px;
  font-weight: 620;
  color: var(--hh-text-primary);
}

.sidebar-config-item-main small {
  font-size: 10px;
  color: var(--hh-text-muted);
}

.sidebar-config-item-controls {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.round-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  font-size: 14px;
  flex-shrink: 0;
}

.round-icon.green {
  background: color-mix(in srgb, #22c55e 15%, transparent);
  color: #22c55e;
}

.round-icon.cyan {
  background: color-mix(in srgb, var(--hh-accent) 15%, transparent);
  color: var(--hh-accent);
}

/* Rail */
.settings-rail {
  display: grid;
  gap: 12px;
  align-content: start;
  min-width: 0;
  min-height: 0;
  max-height: 100%;
  overflow-x: hidden;
  overflow-y: auto;
}

.sidebar-preview-list {
  display: grid;
  gap: 6px;
  list-style: none;
  padding: 0;
  margin: 0;
}

.sidebar-preview-list li {
  display: grid;
  gap: 2px;
}

.sidebar-preview-list li strong {
  font-size: 12px;
  font-weight: 620;
  color: var(--hh-text-primary);
}

.sidebar-preview-list li span {
  font-size: 10px;
  color: var(--hh-text-muted);
}

.detail-list {
  display: grid;
  gap: 6px;
  list-style: none;
  padding: 0;
  margin: 0;
}

.detail-list li {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11px;
  color: var(--hh-text-secondary);
}

.detail-list li em {
  font-size: 9px;
  font-weight: 720;
  color: var(--hh-accent);
  font-style: normal;
  text-transform: uppercase;
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
