import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('YandexTelemostSettingsPanel boundary', () => {
  it('removes the legacy Telemost settings render layer while preserving runtime queries in TS', () => {
    const runtimeQuerySource = readFileSync(
      new URL('../queries/useYandexTelemostRuntimeQuery.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./YandexTelemostSettingsPanel.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../../../shared/yandexTelemost/YandexTelemostSettingsPanelShell.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../../../shared/yandexTelemost/settingsBridge.ts', import.meta.url))).toBe(false)

    expect(runtimeQuerySource).toContain('useYandexTelemostCapabilitiesQuery')
    expect(runtimeQuerySource).toContain('useYandexTelemostRuntimeStatusQuery')
    expect(runtimeQuerySource).toContain('useSetupYandexTelemostAccountMutation')
    expect(runtimeQuerySource).toContain('fetchYandexTelemostCapabilities')
    expect(runtimeQuerySource).toContain('fetchYandexTelemostRuntimeStatus')
    expect(runtimeQuerySource).not.toContain('.vue')
    expect(runtimeQuerySource).not.toContain('<input')
    expect(runtimeQuerySource).not.toContain('<form')
  })
})
