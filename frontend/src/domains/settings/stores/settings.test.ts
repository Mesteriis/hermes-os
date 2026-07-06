import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { readFileSync } from 'node:fs'
import { useNotificationsStore } from '../../../shared/stores/notifications'
import { useSettingsStore } from './settings'

beforeEach(() => {
  setActivePinia(createPinia())
})

describe('settings store notifications', () => {
  it('publishes settings success and error messages through global notifications', () => {
    const settings = useSettingsStore()
    const notifications = useNotificationsStore()

    settings.setActionMessage('AI provider connected')
    settings.setError('API token is required')

    expect(notifications.notificationItems.map((item) => item.title)).toEqual([
      'Settings action failed',
      'Settings action completed',
    ])
    expect(notifications.notificationItems[0]).toMatchObject({
      body: 'API token is required',
      tone: 'danger',
      sourceLabel: 'Settings',
      targetView: 'settings',
      targetId: 'accounts',
    })
    expect(notifications.notificationItems[1]).toMatchObject({
      body: 'AI provider connected',
      tone: 'success',
      sourceLabel: 'Settings',
      targetView: 'settings',
      targetId: 'accounts',
    })
  })

  it('does not render local settings message overlays', () => {
    const source = readFileSync(
      new URL('../views/SettingsPage.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('useToast')
    expect(source).toContain('toast.success')
    expect(source).toContain('toast.error')
    expect(source).not.toContain('class="setup-state success"')
    expect(source).not.toContain('class="inline-error"')
    expect(source).not.toContain('store.actionMessage"')
    expect(source).not.toContain('store.errorMessage"')
  })
})
