<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useProviderAccountsQuery } from '../queries/useSettingsQuery'
import { useSettingsStore } from '../stores/settings'
import { deleteMailAccount, logoutMailAccount, exportMailAccountSettings, importMailAccountSettings } from '../api/settings'
import { useQueryClient } from '@tanstack/vue-query'
import { settingsKeys } from '../queries/useSettingsQuery'
import type { ProviderAccount } from '../types/settings'

const { t } = useI18n()
const store = useSettingsStore()
const queryClient = useQueryClient()
const { data: accountsData } = useProviderAccountsQuery()

const accounts = computed(() => accountsData.value?.items ?? [])

const selectedAccount = computed(() => {
  if (!store.selectedIntegrationId) return null
  return accounts.value.find((a) => a.account_id === store.selectedIntegrationId) ?? null
})

const isImportPanelOpen = ref(false)
const mailImportJson = ref('')
const activeMailAction = ref<string | null>(null)

function providerIcon(providerKind: string): string {
  const icons: Record<string, string> = {
    gmail: 'tabler:mail',
    icloud: 'tabler:cloud',
    imap: 'tabler:server'
  }
  return icons[providerKind] || 'tabler:plug-connected'
}

function providerLabel(providerKind: string): string {
  const labels: Record<string, string> = {
    gmail: 'Gmail',
    icloud: 'iCloud',
    imap: 'IMAP'
  }
  return labels[providerKind] || providerKind
}

function statusLabel(account: ProviderAccount): string {
  if (!account.is_authenticated) return 'Not authenticated'
  if (!account.is_active) return 'Inactive'
  return 'Active'
}

async function handleExport(accountId: string) {
  activeMailAction.value = accountId
  try {
    const result = await exportMailAccountSettings(accountId)
    if (result.result) {
      const filename = `mail-account-${accountId}-${result.result.exported_at}.json`
      downloadJsonFile(filename, JSON.stringify(result.result, null, 2))
      store.setActionMessage(t('Mail account settings exported'))
    }
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Export failed')
  } finally {
    activeMailAction.value = null
  }
}

async function handleLogout(accountId: string) {
  activeMailAction.value = accountId
  try {
    await logoutMailAccount(accountId)
    store.setActionMessage(t('Mail account logged out'))
    queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Logout failed')
  } finally {
    activeMailAction.value = null
  }
}

async function handleDelete(accountId: string) {
  activeMailAction.value = accountId
  try {
    await deleteMailAccount(accountId)
    store.setActionMessage(t('Mail account deleted'))
    queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Delete failed')
  } finally {
    activeMailAction.value = null
  }
}

async function handleImport() {
  try {
    const parsed = JSON.parse(mailImportJson.value)
    await importMailAccountSettings(parsed)
    store.setActionMessage(t('Mail account settings imported'))
    isImportPanelOpen.value = false
    mailImportJson.value = ''
    queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Import failed')
  }
}

function downloadJsonFile(filename: string, content: string) {
  const blob = new Blob([content], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()
  link.remove()
  URL.revokeObjectURL(url)
}
</script>

<template>
  <div class="settings-page">
    <section class="panel settings-list-panel settings-primary-pane">
      <header class="panel-title-row">
        <div>
          <h2>{{ t('Integrations') }}</h2>
          <p>{{ t('Connected accounts and provider services.') }}</p>
        </div>
      </header>

      <!-- Messages -->
      <div v-if="store.actionMessage" class="setup-state success">{{ store.actionMessage }}</div>
      <div v-if="store.errorMessage" class="inline-error">{{ store.errorMessage }}</div>

      <!-- Accounts list -->
      <div v-if="accounts.length === 0" class="empty-panel fill">
        {{ t('No connected accounts yet.') }}
      </div>

      <div v-else class="integrations-table">
        <div
          v-for="account in accounts"
          :key="account.account_id"
          class="integration-row"
          :class="{ selected: store.selectedIntegrationId === account.account_id }"
          @click="store.selectIntegration(account.account_id)"
        >
          <div class="integration-info">
            <span class="integration-icon" v-text="providerIcon(account.provider_kind)" />
            <div>
              <strong>{{ account.label || account.email }}</strong>
              <span class="integration-provider">{{ providerLabel(account.provider_kind) }}</span>
            </div>
          </div>
          <div class="integration-status">
            <span
              class="status-dot"
              :class="{
                active: account.is_authenticated && account.is_active,
                inactive: !account.is_active,
                unauthenticated: !account.is_authenticated
              }"
            />
            <span>{{ statusLabel(account) }}</span>
          </div>
        </div>
      </div>

      <!-- Account inspector (selected) -->
      <div v-if="selectedAccount" class="integration-inspector">
        <header>
          <h3>{{ selectedAccount.label || selectedAccount.email }}</h3>
          <button
            type="button"
            class="hermes-btn hermes-btn--ghost"
            @click="store.selectIntegration(null)"
          >
            {{ t('Close') }}
          </button>
        </header>
        <div class="inspector-details">
          <div class="detail-row">
            <span>{{ t('Provider') }}</span>
            <strong>{{ providerLabel(selectedAccount.provider_kind) }}</strong>
          </div>
          <div class="detail-row">
            <span>{{ t('Email') }}</span>
            <strong>{{ selectedAccount.email }}</strong>
          </div>
          <div class="detail-row">
            <span>{{ t('Status') }}</span>
            <strong>{{ statusLabel(selectedAccount) }}</strong>
          </div>
          <div class="inspector-actions">
            <button
              type="button"
              class="hermes-btn hermes-btn--outline"
              :disabled="activeMailAction === selectedAccount.account_id"
              @click="handleExport(selectedAccount.account_id)"
            >
              {{ t('Export') }}
            </button>
            <button
              type="button"
              class="hermes-btn hermes-btn--outline"
              :disabled="activeMailAction === selectedAccount.account_id"
              @click="handleLogout(selectedAccount.account_id)"
            >
              {{ t('Logout') }}
            </button>
            <button
              type="button"
              class="hermes-btn hermes-btn--destructive"
              :disabled="activeMailAction === selectedAccount.account_id"
              @click="handleDelete(selectedAccount.account_id)"
            >
              {{ t('Delete') }}
            </button>
          </div>
        </div>
      </div>

      <!-- Import panel toggle -->
      <div class="integration-import-section">
        <button
          type="button"
          class="hermes-btn hermes-btn--secondary"
          @click="isImportPanelOpen = !isImportPanelOpen"
        >
          {{ isImportPanelOpen ? t('Cancel') : t('Import Mail Settings') }}
        </button>
        <div v-if="isImportPanelOpen" class="import-panel">
          <textarea
            v-model="mailImportJson"
            class="import-textarea"
            rows="6"
            :placeholder="t('Paste exported mail account JSON here...')"
          />
          <button
            type="button"
            class="hermes-btn hermes-btn--primary"
            :disabled="!mailImportJson.trim()"
            @click="handleImport"
          >
            {{ t('Import') }}
          </button>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.integrations-table {
  display: grid;
  gap: 4px;
  padding: 8px 0;
}

.integration-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: var(--hh-radius-sm);
  cursor: pointer;
  transition: background 100ms ease;
}

.integration-row:hover {
  background: var(--hh-hover-bg);
}

.integration-row.selected {
  background: var(--hh-hover-bg);
  border: 1px solid var(--hh-border);
}

.integration-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.integration-icon {
  font-size: 1.25rem;
  color: var(--hh-text-secondary);
}

.integration-info strong {
  display: block;
  font-size: 13px;
  font-weight: 620;
  color: var(--hh-text-primary);
}

.integration-provider {
  font-size: 11px;
  color: var(--hh-text-muted);
}

.integration-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--hh-text-secondary);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.status-dot.active {
  background: var(--hh-status-success, #22c55e);
}

.status-dot.inactive {
  background: var(--hh-text-muted);
}

.status-dot.unauthenticated {
  background: var(--hh-status-warning, #f59e0b);
}

/* Inspector */
.integration-inspector {
  margin-top: 12px;
  padding: 16px;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  background: var(--hh-surface-deep);
}

.integration-inspector header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.integration-inspector header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 680;
  color: var(--hh-text-primary);
}

.inspector-details {
  display: grid;
  gap: 8px;
}

.detail-row {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
}

.detail-row span {
  color: var(--hh-text-muted);
}

.detail-row strong {
  color: var(--hh-text-primary);
}

.inspector-actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}

/* Import */
.integration-import-section {
  margin-top: 16px;
}

.import-panel {
  margin-top: 8px;
  display: grid;
  gap: 8px;
}

.import-textarea {
  width: 100%;
  padding: 8px;
  background: var(--hh-surface-deep);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-text-primary);
  font-size: 12px;
  font-family: monospace;
  resize: vertical;
  outline: none;
}

.import-textarea:focus-visible {
  box-shadow: 0 0 0 2px var(--hh-focus-ring);
  border-color: var(--hh-accent);
}

.setup-state.success {
  padding: 8px 12px;
  background: color-mix(in srgb, var(--hh-status-success) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--hh-status-success) 30%, transparent);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-status-success);
  font-size: 12px;
  margin-bottom: 8px;
}

.inline-error {
  padding: 8px 12px;
  background: color-mix(in srgb, var(--hh-status-danger) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--hh-status-danger) 30%, transparent);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-status-danger);
  font-size: 12px;
  margin-bottom: 8px;
}
</style>
