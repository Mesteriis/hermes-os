<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from '../../../platform/i18n'

const { t } = useI18n()

type AiTab = 'overview' | 'api-providers' | 'model-routing' | 'prompt-studio' | 'runs-health'

const activeTab = ref<AiTab>('overview')

const tabs: Array<{ id: AiTab; label: string }> = [
  { id: 'overview', label: 'Overview' },
  { id: 'api-providers', label: 'API Providers' },
  { id: 'model-routing', label: 'Model Routing' },
  { id: 'prompt-studio', label: 'Prompt Studio' },
  { id: 'runs-health', label: 'Runs Health' }
]

const statusCards = [
  { label: 'Ollama', status: 'connected', detail: 'v0.5.0' },
  { label: 'Default Model', status: 'configured', detail: 'llama3.2:3b' },
  { label: 'Embeddings', status: 'active', detail: 'nomic-embed-text' },
  { label: 'Last Run', status: 'idle', detail: '2 mins ago' }
]
</script>

<template>
  <div class="settings-page">
    <section class="panel settings-list-panel settings-primary-pane">
      <header class="panel-title-row">
        <div>
          <h2>{{ t('AI Control Center') }}</h2>
          <p>{{ t('Configure AI providers, model routing, prompts and runs.') }}</p>
        </div>
      </header>

      <!-- Tabs -->
      <div class="ai-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          class="ai-tab-btn"
          :class="{ active: activeTab === tab.id }"
          @click="activeTab = tab.id"
        >
          {{ t(tab.label) }}
        </button>
      </div>

      <!-- Overview tab -->
      <div v-if="activeTab === 'overview'" class="ai-tab-content">
        <div class="ai-status-grid">
          <div
            v-for="card in statusCards"
            :key="card.label"
            class="ai-status-card panel"
          >
            <header>
              <span class="ai-status-dot" :class="card.status" />
              <strong>{{ t(card.label) }}</strong>
            </header>
            <p>{{ card.detail }}</p>
          </div>
        </div>

        <section class="ai-section">
          <h3>{{ t('Quick Actions') }}</h3>
          <div class="ai-quick-actions">
            <button type="button" class="hermes-btn hermes-btn--outline" disabled>
              {{ t('Test Connection') }}
            </button>
            <button type="button" class="hermes-btn hermes-btn--outline" disabled>
              {{ t('Run Diagnostics') }}
            </button>
            <button type="button" class="hermes-btn hermes-btn--outline" disabled>
              {{ t('View Logs') }}
            </button>
          </div>
        </section>
      </div>

      <!-- API Providers tab -->
      <div v-else-if="activeTab === 'api-providers'" class="ai-tab-content">
        <div class="empty-panel fill">
          <p>{{ t('AI API provider configuration will be available in a future update.') }}</p>
        </div>
      </div>

      <!-- Model Routing tab -->
      <div v-else-if="activeTab === 'model-routing'" class="ai-tab-content">
        <div class="empty-panel fill">
          <p>{{ t('Model routing configuration will be available in a future update.') }}</p>
        </div>
      </div>

      <!-- Prompt Studio tab -->
      <div v-else-if="activeTab === 'prompt-studio'" class="ai-tab-content">
        <div class="empty-panel fill">
          <p>{{ t('Prompt Studio will be available in a future update.') }}</p>
        </div>
      </div>

      <!-- Runs Health tab -->
      <div v-else-if="activeTab === 'runs-health'" class="ai-tab-content">
        <div class="empty-panel fill">
          <p>{{ t('AI runs health dashboard will be available in a future update.') }}</p>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.ai-tabs {
  display: flex;
  gap: 4px;
  padding: 8px 12px;
  border-top: 1px solid var(--hh-border);
  border-bottom: 1px solid var(--hh-border);
  overflow-x: auto;
}

.ai-tab-btn {
  padding: 6px 14px;
  border: 1px solid transparent;
  border-radius: var(--hh-radius-sm);
  background: transparent;
  color: var(--hh-text-secondary);
  font-size: 12px;
  font-weight: 620;
  cursor: pointer;
  white-space: nowrap;
  transition: all 100ms ease;
}

.ai-tab-btn:hover {
  border-color: var(--hh-border);
  color: var(--hh-text-primary);
}

.ai-tab-btn.active {
  border-color: var(--hh-accent);
  background: var(--hh-accent-tint, color-mix(in srgb, var(--hh-accent) 10%, transparent));
  color: var(--hh-accent);
}

.ai-tab-content {
  padding: 16px;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.ai-status-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 10px;
  margin-bottom: 20px;
}

.ai-status-card {
  padding: 12px;
  border-radius: var(--hh-radius-md);
}

.ai-status-card header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.ai-status-card strong {
  font-size: 13px;
  font-weight: 620;
  color: var(--hh-text-primary);
}

.ai-status-card p {
  font-size: 11px;
  color: var(--hh-text-muted);
  margin: 0;
}

.ai-status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.ai-status-dot.connected,
.ai-status-dot.active {
  background: var(--hh-status-success, #22c55e);
}

.ai-status-dot.configured {
  background: var(--hh-accent);
}

.ai-status-dot.idle {
  background: var(--hh-text-muted);
}

.ai-section {
  margin-top: 16px;
}

.ai-section h3 {
  margin: 0 0 10px;
  font-size: 13px;
  font-weight: 680;
  color: var(--hh-text-primary);
}

.ai-quick-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
</style>
