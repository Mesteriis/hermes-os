import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('AppShell boundary', () => {
  it('uses an app-level surface instead of wiring navigation and theme orchestration inline', () => {
    const source = readFileSync(new URL('./AppShell.vue', import.meta.url), 'utf8')

    expect(source).toContain("import { useAppShellSurface } from '../queries/useAppShellSurface'")
    expect(source).toContain('const { nav, theme } = useAppShellSurface()')
    expect(source).not.toContain('useNavigationStore')
    expect(source).not.toContain('useThemeStore')
    expect(source).not.toContain('useRoute')
    expect(source).not.toContain('theme.hydrateThemeSettings')
    expect(source).not.toContain('nav.syncFromRoute')
  })
})
