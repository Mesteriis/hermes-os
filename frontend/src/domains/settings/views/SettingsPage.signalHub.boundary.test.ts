import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('SettingsPage Signal Hub boundary', () => {
  it('keeps Signal Hub under Settings navigation instead of a standalone route', () => {
    const pageSource = readFileSync(new URL('./SettingsPage.vue', import.meta.url), 'utf8')
    const surfaceSource = readFileSync(new URL('../queries/useSettingsPageSurface.ts', import.meta.url), 'utf8')

    expect(surfaceSource).toContain("id: 'signal-hub'")
    expect(surfaceSource).toContain("label: 'Signal Hub'")
    expect(pageSource).toContain("store.selectedSection === 'signal-hub'")
    expect(pageSource).toContain('Signal Hub contracts are preserved')
    expect(surfaceSource).toContain("id: 'accounts'")
    expect(surfaceSource).toContain("label: 'Accounts'")
    expect(surfaceSource).not.toContain("id: 'integrations'")
    expect(surfaceSource).not.toContain("label: 'Integrations'")
    expect(surfaceSource).not.toContain("path: '/signal-hub'")
    expect(surfaceSource).not.toContain("name: 'signal-hub'")
  })
})
