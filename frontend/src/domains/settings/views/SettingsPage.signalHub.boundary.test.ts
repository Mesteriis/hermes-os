import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('SettingsPage Signal Hub boundary', () => {
  it('keeps Signal Hub under Settings navigation instead of a standalone route', () => {
    const pageSource = readFileSync(new URL('./SettingsPage.vue', import.meta.url), 'utf8')
    const surfaceSource = readFileSync(new URL('../queries/useSettingsPageSurface.ts', import.meta.url), 'utf8')

    expect(surfaceSource).toContain("id: 'signal-hub'")
    expect(surfaceSource).toContain("label: 'Signal Hub'")
    expect(pageSource).toContain("store.selectedSection === 'signal-hub'")
    expect(pageSource).toContain('Signal Hub UI removed after logic extraction. Rebuild pending new design language.')
    expect(surfaceSource).toContain("id: 'integrations'")
    expect(surfaceSource).toContain("label: 'Integrations'")
    expect(surfaceSource).not.toContain("path: '/signal-hub'")
    expect(surfaceSource).not.toContain("name: 'signal-hub'")
  })
})
