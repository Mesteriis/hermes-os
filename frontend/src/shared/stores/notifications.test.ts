import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useNotificationsStore, type NotificationItem } from './notifications'

beforeEach(() => {
  setActivePinia(createPinia())
})

describe('notifications store', () => {
  it('deduplicates notifications by stable action key', () => {
    const store = useNotificationsStore()

    store.addNotification(notification('failed', {
      title: 'Mail action failed',
      body: 'Backend returned 500',
      dedupeKey: 'mail:translation:msg-1',
      tone: 'danger',
    }))
    store.addNotification(notification('ready', {
      title: 'Translation ready',
      body: 'Translated to ru',
      dedupeKey: 'mail:translation:msg-1',
      tone: 'success',
    }))

    expect(store.notificationItems.map((item) => item.id)).toEqual(['ready'])
  })

  it('hides legacy AI helper and transient network mail failures from the drawer', () => {
    const store = useNotificationsStore()

    store.addNotification(notification('ai-helper', {
      title: 'Mail action failed',
      body: '[internal] invalid raw signal event type: signal.raw.ai.message_translation.observed',
      tone: 'danger',
    }))
    store.addNotification(notification('network', {
      title: 'Mail action failed',
      body: '[unknown] Failed to fetch',
      tone: 'danger',
    }))
    store.addNotification(notification('real-failure', {
      title: 'Mail action failed',
      body: 'Provider rejected the command',
      tone: 'danger',
    }))

    expect(store.notificationItems.map((item) => item.id)).toEqual(['real-failure'])
  })

  it('removes notification ids from raw, dismissed and expanded state', () => {
    const store = useNotificationsStore()

    store.addNotification(notification('one'))
    store.addNotification(notification('two'))
    store.dismissNotification('one')
    store.toggleNotificationExpanded('one')

    store.removeNotifications(['one'])

    expect(store.rawNotificationItems.map((item) => item.id)).toEqual(['two'])
    expect(store.dismissedNotificationIds.has('one')).toBe(false)
    expect(store.expandedNotificationIds.has('one')).toBe(false)
  })

  it('clears all notification state including pending targets', () => {
    const store = useNotificationsStore()
    const target = notification('target', {
      targetView: 'communications-mail',
      targetId: 'message-1',
    })

    store.addNotification(target)
    store.addNotification(notification('dismissed'))
    store.dismissNotification('dismissed')
    store.toggleNotificationExpanded('target')
    store.openNotificationTarget(target)

    store.clearNotifications()

    expect(store.rawNotificationItems).toEqual([])
    expect(store.dismissedNotificationIds.size).toBe(0)
    expect(store.expandedNotificationIds.size).toBe(0)
    expect(store.pendingNotificationTarget).toBeNull()
    expect(store.notificationItems).toEqual([])
  })
})

function notification(
  id: string,
  overrides: Partial<NotificationItem> = {}
): NotificationItem {
  return {
    id,
    title: 'Notification',
    body: undefined,
    icon: 'tabler:info-circle',
    tone: 'info',
    sourceLabel: 'Mail',
    time: new Date(`2026-07-06T00:00:0${id.length % 10}.000Z`),
    ...overrides,
  }
}
