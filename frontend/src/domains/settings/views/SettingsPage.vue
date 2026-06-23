<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import { useSettingsStore } from '../stores/settings'
import { useApplicationSettingsQuery } from '../queries/useSettingsQuery'
import Icon from '../../../shared/ui/Icon.vue'
import type { SettingsSection } from '../stores/settings'

import AppearanceSettings from '../components/AppearanceSettings.vue'
import LanguageSettings from '../components/LanguageSettings.vue'
import ApplicationSettings from '../components/ApplicationSettings.vue'
import SidebarSettings from '../components/SidebarSettings.vue'
import IntegrationsSettings from '../components/IntegrationsSettings.vue'
import SignalHubSettings from '../components/SignalHubSettings.vue'
import AISettingsControlCenter from '../components/AISettingsControlCenter.vue'

const { t } = useI18n()
const store = useSettingsStore()
const { data: appSettingsData } = useApplicationSettingsQuery()

const settingsTreeGroups: Array<{ label: string; items: Array<{ id: SettingsSection; label: string; icon: string }> }> = [
  {
    label: 'General',
    items: [
      { id: 'application', label: 'Application', icon: 'tabler:adjustments-horizontal' },
      { id: 'language', label: 'Language', icon: 'tabler:language' }
    ]
  },
  {
    label: 'Interface',
    items: [
      { id: 'appearance', label: 'Appearance', icon: 'tabler:palette' },
      { id: 'sidebar', label: 'Sidebar', icon: 'tabler:layout-sidebar' }
    ]
  },
  {
    label: 'Sources',
    items: [
      { id: 'integrations', label: 'Integrations', icon: 'tabler:plug-connected' },
      { id: 'signal-hub', label: 'Signal Hub', icon: 'tabler:database-import' }
    ]
  },
  {
    label: 'AI',
    items: [
      { id: 'ai', label: 'AI Control Center', icon: 'tabler:sparkles' }
    ]
  }
]

/** Number of provider accounts for the integrations badge */
const integrationCount = appSettingsData.value?.items?.length ?? 0
</script>

<template>
  <div class="settings-page">
    <!-- Action messages -->
    <div v-if="store.actionMessage" class="setup-state success">{{ store.actionMessage }}</div>
    <div v-if="store.errorMessage" class="inline-error">{{ store.errorMessage }}</div>

    <div class="settings-workbench">
      <!-- Navigation Tree -->
      <nav class="settings-tree" :aria-label="t('Settings sections')">
        <section
          v-for="group in settingsTreeGroups"
          :key="group.label"
          class="settings-tree-group"
        >
          <h2>{{ t(group.label) }}</h2>
          <button
            v-for="item in group.items"
            :key="item.id"
            type="button"
            :class="{ active: store.selectedSection === item.id }"
            @click="store.selectSection(item.id)"
          >
            <Icon class="tree-icon" :icon="item.icon" />
            <span>{{ t(item.label) }}</span>
            <em v-if="item.id === 'integrations'">{{ integrationCount }}</em>
          </button>
        </section>
      </nav>

      <!-- Content area -->
      <div class="settings-workbench-content">
        <AppearanceSettings v-if="store.selectedSection === 'appearance'" />
        <LanguageSettings v-else-if="store.selectedSection === 'language'" />
        <ApplicationSettings v-else-if="store.selectedSection === 'application'" />
        <SidebarSettings v-else-if="store.selectedSection === 'sidebar'" />
        <IntegrationsSettings v-else-if="store.selectedSection === 'integrations'" />
        <SignalHubSettings v-else-if="store.selectedSection === 'signal-hub'" />
        <AISettingsControlCenter v-else-if="store.selectedSection === 'ai'" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  display: flex;
  flex-direction: column;
  gap: var(--hh-layout-gap);
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.settings-workbench {
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr);
  gap: var(--hh-layout-gap);
  width: 100%;
  min-width: 0;
  min-height: 0;
  flex: 1;
  overflow: hidden;
}

/* Navigation tree */
.settings-tree {
  display: grid;
  align-content: start;
  gap: 14px;
  min-width: 0;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
  border: 1px solid var(--hh-border-muted);
  border-radius: var(--hh-radius-md);
  background: rgba(4, 18, 20, var(--hh-panel-alpha));
  backdrop-filter: blur(var(--hh-panel-blur));
  box-shadow: var(--hh-shadow-panel);
  padding: 12px 8px;
}

.settings-tree-group {
  display: grid;
  gap: 5px;
}

.settings-tree-group h2 {
  margin: 0;
  color: var(--hh-text-muted);
  font-size: 10px;
  font-weight: 760;
  text-transform: uppercase;
  padding: 0 8px;
}

.settings-tree button {
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr) auto;
  align-items: center;
  gap: 8px;
  min-height: 32px;
  border: 1px solid transparent;
  border-radius: var(--hh-radius-control);
  background: transparent;
  color: var(--hh-text-secondary);
  font-size: 12px;
  font-weight: 650;
  padding: 0 8px;
  text-align: left;
  cursor: pointer;
  transition: all 100ms ease;
}

.settings-tree button:hover,
.settings-tree button:focus-visible {
  border-color: var(--hh-border-accent-soft, var(--hh-accent));
  background: rgba(45, 240, 206, 0.06);
}

.settings-tree button.active {
  border-color: var(--hh-border-accent, var(--hh-accent));
  background: color-mix(in srgb, var(--hh-accent) 10%, transparent);
  color: var(--hh-accent);
}

.settings-tree button span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.settings-tree button em {
  color: var(--hh-text-muted);
  font-size: 10px;
  font-style: normal;
}

.tree-icon {
  width: 14px;
  height: 14px;
  color: currentColor;
}

/* Content */
.settings-workbench-content {
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

/* Messages */
.setup-state.success {
  padding: 8px 12px;
  background: color-mix(in srgb, var(--hh-status-success, #22c55e) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--hh-status-success) 30%, transparent);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-status-success, #22c55e);
  font-size: 12px;
}

.inline-error {
  padding: 8px 12px;
  background: color-mix(in srgb, var(--hh-status-danger) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--hh-status-danger) 30%, transparent);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-status-danger);
  font-size: 12px;
}

/* Responsive */
@media (max-width: 900px) {
  .settings-workbench {
    grid-template-columns: 180px minmax(0, 1fr);
  }
}
</style>
