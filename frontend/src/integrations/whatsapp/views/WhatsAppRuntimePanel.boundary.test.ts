import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

function readSource(relativePath: string): string {
  return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}

describe('WhatsAppRuntimePanel boundary', () => {
  it('surfaces projected sync snapshots for chats, history, members, presence, calls and media through query wiring', () => {
    const source = readSource('./WhatsAppRuntimePanel.vue')
    const snapshotsSource = readSource('../components/WhatsAppRuntimeSnapshots.vue')

    expect(source).toContain('useWhatsappSyncChatsQuery')
    expect(source).toContain('useWhatsappSyncHistoryQuery')
    expect(source).toContain('useWhatsappSyncMembersQuery')
    expect(source).toContain('useWhatsappSyncPresenceQuery(selectedAccountId, selectedSyncChatIdResolved, 8)')
    expect(source).toContain('useWhatsappSyncCallsQuery(selectedAccountId, selectedSyncChatIdResolved, 8)')
    expect(source).toContain('useWhatsappSyncMediaQuery(selectedAccountId, selectedSyncChatIdResolved, 8)')
    expect(snapshotsSource).toContain("Chats")
    expect(snapshotsSource).toContain("History")
    expect(snapshotsSource).toContain("Members")
    expect(snapshotsSource).toContain("Select a synced chat to inspect recent history.")
    expect(snapshotsSource).toContain("Select a synced chat to inspect roster members.")
    expect(snapshotsSource).toContain("No projected presence for the selected synced chat yet.")
    expect(snapshotsSource).toContain("No projected calls for the selected synced chat yet.")
    expect(snapshotsSource).toContain("No projected media for the selected synced chat yet.")
    expect(source).toContain('selectedSyncChatId')
    expect(snapshotsSource).toContain('snapshot-select')
  })

  it('exposes rotate as an owner-visible runtime lifecycle control', () => {
    const source = readSource('./WhatsAppRuntimePanel.vue')
    const controlSource = readSource('../components/WhatsAppRuntimeControl.vue')

    expect(source).toContain('useRotateWhatsappRuntimeMutation')
    expect(controlSource).toContain("emit('set-runtime-state', 'rotate')")
    expect(controlSource).toContain("Rotate")
  })

  it('exposes the owner-visible WebView companion action through the typed Tauri bridge only', () => {
    const source = readSource('./WhatsAppRuntimePanel.vue')
    const controlSource = readSource('../components/WhatsAppRuntimeControl.vue')

    expect(source).toContain("import { openWhatsappWebCompanion } from '../api/whatsappCompanion'")
    expect(source).toContain('async function openVisibleWebCompanion()')
    expect(source).toContain("selectedRuntimeProviderShape.value === 'whatsapp_web_companion'")
    expect(source).toContain("openWhatsappWebCompanion(accountId)")
    expect(controlSource).toContain("Open Companion")
    expect(controlSource).toContain('companionOpenManifest.event_extractor.relay_channel')
    expect(source).not.toContain('window.fetch')
    expect(source).not.toContain('globalThis.fetch')
    expect(source).not.toContain('/api/v1/integrations/whatsapp/runtime-bridge')
    expect(source).not.toContain('ApiClient')
  })

  it('renders nested runtime health diagnostics from backend checks', () => {
    const source = readSource('./WhatsAppRuntimePanel.vue')
    const controlSource = readSource('../components/WhatsAppRuntimeControl.vue')

    expect(source).toContain('runtimeHealthChecks')
    expect(controlSource).toContain('Health diagnostics')
    expect(controlSource).toContain('runtimeHealthCheckStatus')
    expect(controlSource).toContain('runtimeHealthCheckDetail')
    expect(controlSource).toContain('runtimeHealth?.checked_at')
  })
})
