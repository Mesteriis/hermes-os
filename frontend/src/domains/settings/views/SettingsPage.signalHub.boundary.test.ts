import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('SettingsPage Signal Hub boundary', () => {
  it('keeps Signal Hub under Settings navigation instead of a standalone route', () => {
    const source = readFileSync(new URL('./SettingsPage.vue', import.meta.url), 'utf8')

    expect(source).toContain("{ id: 'signal-hub', label: 'Signal Hub'")
    expect(source).toContain("<SignalHubSettings v-else-if=\"store.selectedSection === 'signal-hub'\" />")
    expect(source).toContain("{ id: 'integrations', label: 'Integrations'")
    expect(source).not.toContain("path: '/signal-hub'")
    expect(source).not.toContain("name: 'signal-hub'")
  })
})
