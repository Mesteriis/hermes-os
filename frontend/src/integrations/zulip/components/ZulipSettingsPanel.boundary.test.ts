import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('ZulipSettingsPanel boundary', () => {
  it('removes the legacy Zulip settings render layer while preserving setup and command mutations in TS', () => {
    const runtimeQuerySource = readFileSync(
      new URL('../queries/useZulipRuntimeQuery.ts', import.meta.url),
      'utf8'
    )
    const bridgeSource = readFileSync(
      new URL('../../../shared/zulip/settingsBridge.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./ZulipSettingsPanel.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../../../shared/zulip/ZulipSettingsPanelShell.vue', import.meta.url))).toBe(false)

    expect(runtimeQuerySource).toContain('useSetupZulipBotAccountMutation')
    expect(runtimeQuerySource).toContain('useEnqueueZulipStreamUploadCommandMutation')
    expect(runtimeQuerySource).toContain('useEnqueueZulipDirectUploadCommandMutation')
    expect(runtimeQuerySource).toContain('useEnqueueZulipUploadCommandMutation')
    expect(runtimeQuerySource).toContain('settingsKeys.providerAccounts()')
    expect(runtimeQuerySource).toContain('settingsKeys.workspace()')
    expect(runtimeQuerySource).not.toContain('.vue')
    expect(runtimeQuerySource).not.toContain('<form')
    expect(runtimeQuerySource).not.toContain('<input')

    expect(bridgeSource).toContain('settingsKeys')
  })
})
