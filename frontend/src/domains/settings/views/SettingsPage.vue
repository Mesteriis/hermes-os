<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useSettingsPageSurface } from '../queries/useSettingsPageSurface'

const { t } = useI18n()
const {
  realtimeStatus,
  settingsOverviewCards,
  settingsTreeGroups,
  selectedTreeItem,
  store,
} = useSettingsPageSurface()
</script>

<template>
  <div class="settings-page">
    <div v-if="store.actionMessage" class="setup-state success">{{ store.actionMessage }}</div>
    <div v-if="store.errorMessage" class="inline-error">{{ store.errorMessage }}</div>

    <section class="settings-console-overview" :aria-label="t('Settings overview')">
      <article
        v-for="card in settingsOverviewCards"
        :key="card.id"
        class="settings-overview-card"
        :class="`tone-${card.tone}`"
      >
        <span class="settings-overview-icon">
          <Icon :icon="card.icon" />
        </span>
        <div class="settings-overview-copy">
          <span>{{ t(card.label) }}</span>
          <strong>{{ card.value }}</strong>
          <small>{{ card.detail }}</small>
        </div>
        <button
          v-if="card.id === 'realtime' && realtimeStatus.canTriggerReconnect"
          type="button"
          class="settings-overview-action"
          @click="realtimeStatus.requestReconnect()"
        >
          {{ t('Reconnect') }}
        </button>
      </article>
    </section>

    <div class="settings-workbench">
      <nav class="settings-tree" :aria-label="t('Settings sections')">
        <header class="settings-tree-header">
          <span>{{ t('Control Center') }}</span>
          <strong>{{ t('Local-first system settings') }}</strong>
        </header>

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
            <span class="settings-tree-copy">
              <strong>{{ t(item.label) }}</strong>
              <small>{{ t(item.description) }}</small>
            </span>
            <em v-if="item.meta">{{ item.meta }}</em>
          </button>
        </section>
      </nav>

      <div class="settings-workbench-content">
        <header v-if="selectedTreeItem" class="settings-section-header">
          <div>
            <span>{{ t('Settings') }}</span>
            <h2>{{ t(selectedTreeItem.label) }}</h2>
            <p>{{ t(selectedTreeItem.description) }}</p>
          </div>
        </header>

        <section
          v-if="store.selectedSection === 'appearance'"
          class="panel settings-list-panel settings-primary-pane settings-placeholder-panel"
        >
          <header class="panel-title-row">
            <div>
              <h2>{{ t('Appearance') }}</h2>
              <p>{{ t('Appearance UI removed after logic extraction. Rebuild pending new design language.') }}</p>
            </div>
          </header>
          <div class="settings-placeholder-copy">
            <strong>{{ t('Appearance logic is preserved') }}</strong>
            <p>{{ t('Theme persistence, preview state and shell preference flows now live outside Vue components. This section stays intentionally empty until the new render layer is rebuilt.') }}</p>
          </div>
        </section>
        <section
          v-else-if="store.selectedSection === 'language'"
          class="panel settings-list-panel settings-primary-pane settings-placeholder-panel"
        >
          <header class="panel-title-row">
            <div>
              <h2>{{ t('Language') }}</h2>
              <p>{{ t('Language UI removed after logic extraction. Rebuild pending new design language.') }}</p>
            </div>
          </header>
          <div class="settings-placeholder-copy">
            <strong>{{ t('Language logic is preserved') }}</strong>
            <p>{{ t('Locale switching and persistence now live in a dedicated settings surface. This section stays intentionally empty until the new render layer is rebuilt.') }}</p>
          </div>
        </section>
        <section
          v-else-if="store.selectedSection === 'application'"
          class="panel settings-list-panel settings-primary-pane settings-placeholder-panel"
        >
          <header class="panel-title-row">
            <div>
              <h2>{{ t('Application Settings') }}</h2>
              <p>{{ t('Application settings UI removed after logic extraction. Rebuild pending new design language.') }}</p>
            </div>
          </header>
          <div class="settings-placeholder-copy">
            <strong>{{ t('Application settings logic is preserved') }}</strong>
            <p>{{ t('Settings registry, draft coercion and save orchestration now live in a dedicated TypeScript surface. This section stays intentionally empty until the new render layer is rebuilt.') }}</p>
          </div>
        </section>
        <section
          v-else-if="store.selectedSection === 'sidebar'"
          class="panel settings-list-panel settings-primary-pane settings-placeholder-panel"
        >
          <header class="panel-title-row">
            <div>
              <h2>{{ t('Sidebar') }}</h2>
              <p>{{ t('Sidebar UI removed after logic extraction. Rebuild pending new design language.') }}</p>
            </div>
          </header>
          <div class="settings-placeholder-copy">
            <strong>{{ t('Sidebar logic is preserved') }}</strong>
            <p>{{ t('Sidebar grouping, hidden-state rules and persistence now live outside Vue components. This section stays intentionally empty until the new render layer is rebuilt.') }}</p>
          </div>
        </section>
        <section
          v-else-if="store.selectedSection === 'integrations'"
          class="panel settings-list-panel settings-primary-pane settings-placeholder-panel"
        >
          <header class="panel-title-row">
            <div>
              <h2>{{ t('Integrations') }}</h2>
              <p>{{ t('Component removed after logic extraction. Rebuild will land in the next UI pass.') }}</p>
            </div>
          </header>
          <div class="settings-placeholder-copy">
            <strong>{{ t('Integration logic is preserved') }}</strong>
            <p>{{ t('Provider queries, connection routing and wizard state now live outside Vue components. This screen stays intentionally empty until the new render layer is rebuilt.') }}</p>
          </div>
        </section>
        <section
          v-else-if="store.selectedSection === 'signal-hub'"
          class="panel settings-list-panel settings-primary-pane settings-placeholder-panel"
        >
          <header class="panel-title-row">
            <div>
              <h2>{{ t('Signal Hub') }}</h2>
              <p>{{ t('Signal Hub UI removed after logic extraction. Rebuild pending new design language.') }}</p>
            </div>
          </header>
          <div class="settings-placeholder-copy">
            <strong>{{ t('Signal Hub logic is preserved') }}</strong>
            <p>{{ t('Profiles, replay workflows, runtime controls and health diagnostics now live in TypeScript surfaces. This section stays intentionally empty until the new render layer is rebuilt.') }}</p>
          </div>
        </section>
        <section
          v-else-if="store.selectedSection === 'ai'"
          class="panel settings-list-panel settings-primary-pane settings-placeholder-panel"
        >
          <header class="panel-title-row">
            <div>
              <h2>{{ t('AI Control Center') }}</h2>
              <p>{{ t('AI settings UI removed after logic extraction. Rebuild pending new design language.') }}</p>
            </div>
          </header>
          <div class="settings-placeholder-copy">
            <strong>{{ t('Legacy AI control center removed') }}</strong>
            <p>{{ t('The previous AI control center was only a static placeholder. It has been removed so the next render layer can start clean.') }}</p>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

