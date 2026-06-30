import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('NotificationsDrawer boundary', () => {
  it('delegates notification lookup and preview formatting to an app-level surface', () => {
    const source = readFileSync(new URL('./NotificationsDrawer.vue', import.meta.url), 'utf8')

    expect(source).toContain("import { useNotificationsDrawerSurface } from '../queries/useNotificationsDrawerSurface'")
    expect(source).toContain('notificationBodyPreview(notification)')
    expect(source).toContain('canExpandNotification(notification)')
    expect(source).not.toContain('useNotificationsStore')
    expect(source).not.toContain('useNavigationStore')
    expect(source).not.toContain('notification.body.slice')
    expect(source).not.toContain('notification.body.length > 120')
    expect(source).not.toContain('notifications.notificationItems.find')
    expect(source).not.toContain('date.toLocaleString()')
  })
})
