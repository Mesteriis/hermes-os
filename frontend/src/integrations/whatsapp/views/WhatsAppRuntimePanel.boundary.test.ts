import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('WhatsAppRuntimePanel boundary', () => {
  it('surfaces projected sync snapshots for chats, history, members, presence, calls and media through query wiring', () => {
    const source = readFileSync(new URL('./WhatsAppRuntimePanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('useWhatsappSyncChatsQuery')
    expect(source).toContain('useWhatsappSyncHistoryQuery')
    expect(source).toContain('useWhatsappSyncMembersQuery')
    expect(source).toContain('useWhatsappSyncPresenceQuery(selectedAccountId, selectedSyncChatIdResolved, 8)')
    expect(source).toContain('useWhatsappSyncCallsQuery(selectedAccountId, selectedSyncChatIdResolved, 8)')
    expect(source).toContain('useWhatsappSyncMediaQuery(selectedAccountId, selectedSyncChatIdResolved, 8)')
    expect(source).toContain("Chats")
    expect(source).toContain("History")
    expect(source).toContain("Members")
    expect(source).toContain("Select a synced chat to inspect recent history.")
    expect(source).toContain("Select a synced chat to inspect roster members.")
    expect(source).toContain("No projected presence for the selected synced chat yet.")
    expect(source).toContain("No projected calls for the selected synced chat yet.")
    expect(source).toContain("No projected media for the selected synced chat yet.")
    expect(source).toContain('selectedSyncChatId')
    expect(source).toContain('snapshot-select')
  })

  it('exposes rotate as an owner-visible runtime lifecycle control', () => {
    const source = readFileSync(new URL('./WhatsAppRuntimePanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('useRotateWhatsappRuntimeMutation')
    expect(source).toContain("setRuntimeState('rotate')")
    expect(source).toContain("Rotate")
  })

  it('exposes the owner-visible WebView companion action through the typed Tauri bridge only', () => {
    const source = readFileSync(new URL('./WhatsAppRuntimePanel.vue', import.meta.url), 'utf8')

    expect(source).toContain("import { openWhatsappWebCompanion } from '../api/whatsappCompanion'")
    expect(source).toContain('async function openVisibleWebCompanion()')
    expect(source).toContain("selectedRuntimeProviderShape.value === 'whatsapp_web_companion'")
    expect(source).toContain("openWhatsappWebCompanion(accountId)")
    expect(source).toContain("Open Companion")
    expect(source).toContain('companionOpenManifest.event_extractor.relay_channel')
    expect(source).not.toContain('window.fetch')
    expect(source).not.toContain('globalThis.fetch')
    expect(source).not.toContain('/api/v1/integrations/whatsapp/runtime-bridge')
    expect(source).not.toContain('ApiClient')
  })

  it('renders nested runtime health diagnostics from backend checks', () => {
    const source = readFileSync(new URL('./WhatsAppRuntimePanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('runtimeHealthChecks')
    expect(source).toContain('Health diagnostics')
    expect(source).toContain('runtimeHealthCheckStatus')
    expect(source).toContain('runtimeHealthCheckDetail')
    expect(source).toContain('runtimeHealth?.checked_at')
  })
})
