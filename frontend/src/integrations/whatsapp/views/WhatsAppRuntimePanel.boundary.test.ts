import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

function readSource(relativePath: string): string {
  return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}

describe('WhatsAppRuntimePanel boundary', () => {
  it('preserves projected sync snapshot orchestration after removing the runtime panel Vue layer', () => {
    const surfaceSource = readSource('../queries/useWhatsappRuntimePanelSurface.ts')
    const presentationSource = readSource('../queries/useWhatsappRuntimePresentation.ts')

    expect(existsSync(new URL('./WhatsAppRuntimePanel.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/WhatsAppRuntimeAccountList.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/WhatsAppRuntimeCapabilities.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/WhatsAppRuntimeCommandAudit.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/WhatsAppRuntimeControl.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/WhatsAppRuntimeLinking.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/WhatsAppRuntimeSnapshots.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/WhatsAppSessionList.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/WhatsAppStatusMessages.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/WhatsAppRail.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./WhatsAppRuntimePanel.helpers.ts', import.meta.url))).toBe(false)

    expect(surfaceSource).toContain('useWhatsappSyncChatsQuery')
    expect(surfaceSource).toContain('useWhatsappSyncHistoryQuery')
    expect(surfaceSource).toContain('useWhatsappSyncMembersQuery')
    expect(surfaceSource).toContain('useWhatsappSyncPresenceQuery(')
    expect(surfaceSource).toContain('useWhatsappSyncCallsQuery(')
    expect(surfaceSource).toContain('useWhatsappSyncMediaQuery(')
    expect(surfaceSource).toContain('selectedSyncChatId')
    expect(surfaceSource).toContain('statusPublishText')
    expect(surfaceSource).toContain('publishStatus')
    expect(surfaceSource).toContain('selectWhatsappSession')
    expect(surfaceSource).not.toContain('.vue')

    expect(presentationSource).toContain('chatLabel')
    expect(presentationSource).toContain('chatMeta')
    expect(presentationSource).toContain('historyLabel')
    expect(presentationSource).toContain('statusPreview')
    expect(presentationSource).toContain('presenceLabel')
    expect(presentationSource).toContain('callLabel')
    expect(presentationSource).toContain('contactLabel')
    expect(presentationSource).toContain('mediaLabel')
    expect(presentationSource).toContain('memberLabel')
    expect(presentationSource).toContain('snapshotTimestamp')
    expect(presentationSource).not.toContain('.vue')
  })

  it('exposes rotate as a hidden-runtime lifecycle control', () => {
    const surfaceSource = readSource('../queries/useWhatsappRuntimePanelSurface.ts')

    expect(surfaceSource).toContain('useRotateWhatsappRuntimeMutation')
    expect(surfaceSource).toContain("async function setRuntimeState(action: 'start' | 'stop' | 'revoke' | 'relink' | 'rotate' | 'remove')")
    expect(surfaceSource).toContain("} else if (action === 'rotate') {")
  })

  it('exposes the hidden WebView runtime action through the typed mutation surface only', () => {
    const surfaceSource = readSource('../queries/useWhatsappRuntimePanelSurface.ts')

    expect(surfaceSource).toContain('useStartHiddenWhatsappWebviewMutation')
    expect(surfaceSource).toContain('async function startHiddenWebview()')
    expect(surfaceSource).toContain("selectedRuntimeProviderShape.value === 'whatsapp_web_companion'")
    expect(surfaceSource).toContain('hiddenWebviewMutation.mutateAsync')
    expect(surfaceSource).toContain('hiddenWebviewMutation.isPending.value')
    expect(surfaceSource).toContain('companionOpenManifest.value = manifest')
    expect(surfaceSource).toContain('Hidden WebView runtime is available only')
    expect(surfaceSource).not.toContain('window.fetch')
    expect(surfaceSource).not.toContain('globalThis.fetch')
    expect(surfaceSource).not.toContain('/api/v1/integrations/whatsapp/runtime-bridge')
    expect(surfaceSource).not.toContain('ApiClient')
  })

  it('does not leave manual provisioning primitives in the runtime surface artifacts', () => {
    const surfaceSource = readSource('../queries/useWhatsappRuntimePanelSurface.ts')

    expect(surfaceSource).not.toContain('useSetupWhatsappLiveAccountMutation')
    expect(surfaceSource).not.toContain('useStartWhatsappQrLinkMutation')
    expect(surfaceSource).not.toContain('useStartWhatsappPairCodeLinkMutation')
    expect(surfaceSource).not.toContain('manual_dead_letter_from_runtime_panel')
  })

  it('keeps QR-based capability label filtering out of the preserved runtime artifacts', () => {
    const surfaceSource = readSource('../queries/useWhatsappRuntimePanelSurface.ts')

    expect(surfaceSource).not.toContain('auth.qr_link_start')
    expect(surfaceSource).not.toContain('auth.pair_code_link_start')
  })

  it('preserves nested runtime health and command presentation helpers in TS', () => {
    const surfaceSource = readSource('../queries/useWhatsappRuntimePanelSurface.ts')
    const presentationSource = readSource('../queries/useWhatsappRuntimePresentation.ts')

    expect(surfaceSource).toContain('runtimeHealthChecks')
    expect(surfaceSource).toContain('providerCommands')
    expect(surfaceSource).toContain('retryCommand')
    expect(surfaceSource).toContain('deadLetterCommand')
    expect(presentationSource).toContain('runtimeHealthCheckStatus')
    expect(presentationSource).toContain('runtimeHealthCheckDetail')
    expect(presentationSource).toContain('commandStatusTone')
    expect(presentationSource).toContain('canRetryCommand')
    expect(presentationSource).toContain('canDeadLetterCommand')
    expect(presentationSource).toContain('providerTargetLabel')
  })
})
