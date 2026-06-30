import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('AISettingsControlCenter boundary', () => {
  it('removes the legacy AI placeholder render layer from SettingsPage', () => {
    const page = readFileSync(
      new URL('../views/SettingsPage.vue', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./AISettingsControlCenter.vue', import.meta.url))).toBe(false)
    expect(page).not.toContain('import AISettingsControlCenter')
    expect(page).not.toContain('<AISettingsControlCenter')
    expect(page).toContain('AI settings UI removed after logic extraction. Rebuild pending new design language.')
    expect(page).toContain('The previous AI control center was only a static placeholder. It has been removed so the next render layer can start clean.')
  })
})
