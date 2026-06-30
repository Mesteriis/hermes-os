import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('Topbar boundary', () => {
  it('delegates realtime/icon/locale wiring to an app-level surface', () => {
    const source = readFileSync(new URL('./Topbar.vue', import.meta.url), 'utf8')

    expect(source).toContain("import { useTopbarSurface } from '../queries/useTopbarSurface'")
    expect(source).toContain('const {')
    expect(source).toContain('notificationBadgeLabel')
    expect(source).toContain('localeToggleLabel')
    expect(source).not.toContain('useNavigationStore')
    expect(source).not.toContain('useNotificationsStore')
    expect(source).not.toContain('useRealtimeStatusStore')
    expect(source).not.toContain('setLocale(locale.value ===')
    expect(source).not.toContain('window.close()')
    expect(source).not.toContain("return 'tabler:cloud-check'")
  })
})
