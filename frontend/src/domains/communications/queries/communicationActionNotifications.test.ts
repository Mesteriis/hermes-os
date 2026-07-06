import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useNotificationsStore } from '@/shared/stores/notifications'
import { useCommunicationActionNotifications } from './communicationActionNotifications'

beforeEach(() => {
  setActivePinia(createPinia())
})

describe('communication action notifications', () => {
  it('does not persist transient backend fetch failures in the notification drawer', () => {
    const store = useNotificationsStore()
    const notifications = useCommunicationActionNotifications()

    notifications.error(
      'Mail action failed',
      'Backend API is unavailable. Check system health and retry.',
      'msg-1',
      'mail:translation:msg-1'
    )

    expect(store.notificationItems).toEqual([])
  })

  it('prunes obsolete AI helper signal failures when publishing a new mail notification', () => {
    const store = useNotificationsStore()
    const notifications = useCommunicationActionNotifications()

    store.rawNotificationItems = [
      {
        id: 'legacy-ai-helper',
        title: 'Mail action failed',
        body: '[internal] invalid raw signal event type: signal.raw.ai.message_translation.observed',
        icon: 'tabler:alert-circle',
        tone: 'danger',
        sourceLabel: 'Mail',
        time: new Date('2026-07-06T00:00:00.000Z'),
        targetView: 'communications-mail',
        targetId: 'msg-1',
      },
    ]

    notifications.success('Translation ready', 'Translated to ru', 'msg-1', 'mail:translation:msg-1')

    expect(store.rawNotificationItems.map((item) => item.title)).toEqual(['Translation ready'])
    expect(store.notificationItems.map((item) => item.title)).toEqual(['Translation ready'])
  })
})
