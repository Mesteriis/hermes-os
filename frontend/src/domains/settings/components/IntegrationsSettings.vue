<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { settingsKeys, useProviderAccountsQuery } from '../queries/useSettingsQuery'
import { useSettingsStore } from '../stores/settings'
import { deleteMailAccount, exportMailAccountSettings, importMailAccountSettings, logoutMailAccount } from '../api/settings'
import type { ProviderAccount } from '../types/settings'
import { useQueryClient } from '@tanstack/vue-query'
import ZoomSettingsPanelShell from '../../../shared/zoom/ZoomSettingsPanelShell.vue'
type MailProviderKind = 'gmail' | 'icloud' | 'imap'

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

function isMailProvider(providerKind: string): boolean {
  const mailProviders: MailProviderKind[] = ['gmail', 'icloud', 'imap']
  return mailProviders.includes(providerKind as MailProviderKind)
}

function isZoomProvider(providerKind: string): boolean {
  return providerKind === 'zoom_user' || providerKind === 'zoom_server_to_server'
}

const groups = computed(() => {
  const mail = accounts.value.filter((a) => isMailProvider(a.provider_kind))
  const zoom = accounts.value.filter((a) => isZoomProvider(a.provider_kind))
  const other = accounts.value.filter((a) => !isMailProvider(a.provider_kind) && !isZoomProvider(a.provider_kind))
  const rows = []
  if (mail.length) rows.push({ label: t('Mail accounts'), items: mail })
  if (zoom.length) rows.push({ label: t('Zoom accounts'), items: zoom })
  if (other.length) rows.push({ label: t('Other accounts'), items: other })
  if (!rows.length) rows.push({ label: t('Accounts'), items: accounts.value })
  return rows
})

function providerIcon(providerKind: string): string {
  const icons: Record<string, string> = {
    gmail: 'tabler:mail',
    icloud: 'tabler:cloud',
    imap: 'tabler:server',
    zoom_user: 'tabler:video',
    zoom_server_to_server: 'tabler:video-plus',
  }
  return icons[providerKind] || 'tabler:plug-connected'
}

function providerLabel(providerKind: string): string {
  const labels: Record<string, string> = {
    gmail: 'Gmail',
    icloud: 'iCloud',
    imap: 'IMAP',
    zoom_user: 'Zoom (OAuth/Live)',
    zoom_server_to_server: 'Zoom (Server-to-Server)',
  }
  return labels[providerKind] || providerKind
}

function providerDisplayName(account: ProviderAccount): string {
  return (
    account.display_name ||
    account.label ||
    account.email ||
    (typeof account.config?.email === 'string' ? account.config.email : null) ||
    account.external_account_id ||
    account.account_id
  )
}

function statusText(account: ProviderAccount): string {
  if (typeof account.is_authenticated === 'boolean' && !account.is_authenticated) return t('Not authenticated')
  if (typeof account.is_active === 'boolean' && !account.is_active) return t('Inactive')
  if (isZoomProvider(account.provider_kind)) return t('Configured')
  return t('Active')
}

function statusClass(account: ProviderAccount) {
  if (typeof account.is_authenticated === 'boolean' && !account.is_authenticated) return 'unauthenticated'
  if (typeof account.is_active === 'boolean' && !account.is_active) return 'inactive'
  if (isZoomProvider(account.provider_kind)) return 'configured'
  return 'active'
}

function selectedAccountEmail(account: ProviderAccount): string {
  return account.email || (typeof account.config?.email === 'string' ? account.config.email : '')
}

async function refreshSettings() {
  await queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
}

async function handleExport(accountId: string) {
  activeMailAction.value = accountId
  try {
    const result = await exportMailAccountSettings(accountId)
    if (result.result) {
      const filename = `mail-account-${accountId}-${result.result.exported_at}.json`
      const blob = new Blob([JSON.stringify(result.result, null, 2)], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      link.download = filename
      document.body.appendChild(link)
      link.click()
      link.remove()
      URL.revokeObjectURL(url)
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
    await refreshSettings()
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
    await refreshSettings()
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
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Import failed')
  }
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
      <div v-if="store.actionMessage" class="setup-state success">{{ store.actionMessage }}</div>
      <div v-if="store.errorMessage" class="inline-error">{{ store.errorMessage }}</div>

      <div v-if="accounts.length === 0" class="empty-panel fill">{{ t('No connected accounts yet.') }}</div>
      <div v-else v-for="group in groups" :key="group.label" class="integration-group">
        <h3>{{ group.label }}</h3>
        <div class="integrations-table">
          <button
            v-for="account in group.items"
            :key="account.account_id"
            type="button"
            class="integration-row"
            :class="{ selected: store.selectedIntegrationId === account.account_id }"
            @click="store.selectIntegration(account.account_id)"
          >
            <div class="integration-info">
              <span class="integration-icon" v-text="providerIcon(account.provider_kind)" />
              <div>
                <strong>{{ providerDisplayName(account) }}</strong>
                <span class="integration-provider">{{ providerLabel(account.provider_kind) }}</span>
              </div>
            </div>
            <div class="integration-status">
              <span class="status-dot" :class="statusClass(account)" />
              <span>{{ statusText(account) }}</span>
            </div>
          </button>
        </div>
      </div>

      <div v-if="selectedAccount" class="integration-inspector">
        <header>
          <h3>{{ providerDisplayName(selectedAccount) }}</h3>
          <button type="button" class="hermes-btn hermes-btn--ghost" @click="store.selectIntegration(null)">
            {{ t('Close') }}
          </button>
        </header>

        <div class="inspector-details">
          <div class="detail-row"><span>{{ t('Provider') }}</span><strong>{{ providerLabel(selectedAccount.provider_kind) }}</strong></div>
          <div class="detail-row"><span>{{ t('External account id') }}</span><strong>{{ selectedAccount.external_account_id }}</strong></div>
          <div v-if="selectedAccountEmail(selectedAccount)" class="detail-row">
            <span>{{ t('Email') }}</span><strong>{{ selectedAccountEmail(selectedAccount) }}</strong>
          </div>
          <div class="detail-row"><span>{{ t('Status') }}</span><strong>{{ statusText(selectedAccount) }}</strong></div>
        </div>

        <div v-if="isMailProvider(selectedAccount.provider_kind)" class="inspector-actions">
          <button type="button" class="hermes-btn hermes-btn--outline" :disabled="activeMailAction===selectedAccount.account_id" @click="handleExport(selectedAccount.account_id)">
            {{ t('Export') }}
          </button>
          <button type="button" class="hermes-btn hermes-btn--outline" :disabled="activeMailAction===selectedAccount.account_id" @click="handleLogout(selectedAccount.account_id)">
            {{ t('Logout') }}
          </button>
          <button type="button" class="hermes-btn hermes-btn--destructive" :disabled="activeMailAction===selectedAccount.account_id" @click="handleDelete(selectedAccount.account_id)">
            {{ t('Delete') }}
          </button>
        </div>

      </div>

      <ZoomSettingsPanelShell :selected-account="selectedAccount" @removed="store.selectIntegration(null)" />

      <div class="integration-import-section">
        <button type="button" class="hermes-btn hermes-btn--secondary" @click="isImportPanelOpen = !isImportPanelOpen">
          {{ isImportPanelOpen ? t('Cancel') : t('Import Mail Settings') }}
        </button>
        <div v-if="isImportPanelOpen" class="import-panel">
          <textarea v-model="mailImportJson" class="import-textarea" rows="6" :placeholder="t('Paste exported mail account JSON here...')" />
          <button type="button" class="hermes-btn hermes-btn--primary" :disabled="!mailImportJson.trim()" @click="handleImport">
            {{ t('Import') }}
          </button>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.integration-group { margin-top: 10px; border: 1px solid var(--hh-border); border-radius: var(--hh-radius-md); background: var(--hh-surface-deep); padding: 10px; }
.integration-group h3,.integration-section h3 { margin: 0 0 6px; font-size: 13px; color: var(--hh-text-secondary); }
.integrations-table { display: grid; gap: 4px; }
.integration-row { display: flex; align-items: center; justify-content: space-between; padding: 10px 12px; border-radius: var(--hh-radius-sm); cursor: pointer; transition: background 100ms ease; }
.integration-row:hover, .integration-row.selected { background: var(--hh-hover-bg); }
.integration-row.selected { border: 1px solid var(--hh-border); }
.integration-info { display: flex; align-items: center; gap: 10px; }
.integration-icon { font-size: 1.25rem; color: var(--hh-text-secondary); }
.integration-provider { font-size: 11px; color: var(--hh-text-muted); }
.integration-status { display: flex; align-items: center; gap: 6px; font-size: 11px; color: var(--hh-text-secondary); }
.status-dot { width: 8px; height: 8px; border-radius: 50%; }
.status-dot.active { background: var(--hh-status-success, #22c55e); }
.status-dot.inactive { background: var(--hh-text-muted); }
.status-dot.unauthenticated { background: var(--hh-status-warning, #f59e0b); }
.status-dot.configured { background: var(--hh-text-muted); }
.integration-inspector { margin-top: 12px; padding: 16px; border: 1px solid var(--hh-border); border-radius: var(--hh-radius-md); background: var(--hh-surface-deep); }
.integration-inspector header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
.integration-inspector h3 { margin: 0; font-size: 14px; }
.integration-inspector h4 { margin: 0; color: var(--hh-text-secondary); font-size: 12px; }
.inspector-details { display: grid; gap: 8px; }
.detail-row { display: flex; justify-content: space-between; font-size: 12px; }
.detail-row span { color: var(--hh-text-muted); }
.detail-row strong { color: var(--hh-text-primary); }
.inspector-actions { display: flex; gap: 8px; margin-top: 12px; }
.integration-section { margin-top: 12px; border: 1px solid var(--hh-border); border-radius: var(--hh-radius-md); background: var(--hh-surface-deep); padding: 12px; }
.integration-section-description { margin: 0 0 8px; font-size: 12px; color: var(--hh-text-muted); }
.integration-form { display: grid; gap: 8px; }
.integration-form label { display: grid; gap: 4px; font-size: 11px; color: var(--hh-text-muted); }
.integration-form button { margin-top: 6px; }
.integration-import-section { margin-top: 16px; }
.import-panel { margin-top: 8px; display: grid; gap: 8px; }
.import-textarea { width: 100%; padding: 8px; background: var(--hh-surface-deep); border: 1px solid var(--hh-border); border-radius: var(--hh-radius-sm); color: var(--hh-text-primary); font-size: 12px; font-family: monospace; resize: vertical; outline: none; }
.import-textarea:focus-visible { box-shadow: 0 0 0 2px var(--hh-focus-ring); border-color: var(--hh-accent); }
.setup-state.success { padding: 8px 12px; background: color-mix(in srgb, var(--hh-status-success) 15%, transparent); border: 1px solid color-mix(in srgb, var(--hh-status-success) 30%, transparent); border-radius: var(--hh-radius-sm); color: var(--hh-status-success); font-size: 12px; margin-bottom: 8px; }
.inline-error { padding: 8px 12px; background: color-mix(in srgb, var(--hh-status-danger) 15%, transparent); border: 1px solid color-mix(in srgb, var(--hh-status-danger) 30%, transparent); border-radius: var(--hh-radius-sm); color: var(--hh-status-danger); font-size: 12px; margin-bottom: 8px; }
</style>
