import { getActivePinia } from 'pinia'
import {
  useNotificationsStore,
  type NotificationItem,
} from '@/shared/stores/notifications'
import { useToast } from '@/shared/ui'

type CommunicationActionNotificationTone =
  | 'info'
  | 'success'
  | 'warning'
  | 'danger'

type CommunicationActionNotification = {
  title: string
  body?: string
  tone?: CommunicationActionNotificationTone
  icon?: string
  targetId?: string
  dedupeKey?: string
}

let notificationCounter = 0

export function useCommunicationActionNotifications(sourceLabel = 'Mail') {
  const toast = useToast()
  const notificationsStore = getActivePinia() ? useNotificationsStore() : null

  function publish(notification: CommunicationActionNotification): void {
    const tone = notification.tone ?? 'info'
    const id = `communication-action-${Date.now()}-${++notificationCounter}`
    const item: NotificationItem = {
      id,
      title: notification.title,
      body: notification.body,
      icon: notification.icon ?? notificationIcon(tone),
      tone,
      sourceLabel,
      time: new Date(),
      targetView: 'communications-mail',
      targetId: notification.targetId,
      dedupeKey: notification.dedupeKey,
    }

    removeObsoleteCommunicationNotifications()

    if (!isTransientCommunicationFailure(item)) {
      notificationsStore?.addNotification(item)
    }

    toast.addToast({
      title: notification.title,
      description: notification.body,
      variant: notificationToastVariant(tone),
      duration: 5000,
    })
  }

  return {
    publish,
    info: (title: string, body?: string, targetId?: string, dedupeKey?: string) =>
      publish({ title, body, targetId, dedupeKey, tone: 'info' }),
    success: (title: string, body?: string, targetId?: string, dedupeKey?: string) =>
      publish({ title, body, targetId, dedupeKey, tone: 'success' }),
    warning: (title: string, body?: string, targetId?: string, dedupeKey?: string) =>
      publish({ title, body, targetId, dedupeKey, tone: 'warning' }),
    error: (title: string, body?: string, targetId?: string, dedupeKey?: string) =>
      publish({ title, body, targetId, dedupeKey, tone: 'danger' }),
  }

  function removeObsoleteCommunicationNotifications(): void {
    const ids = notificationsStore?.rawNotificationItems
      .filter(isObsoleteCommunicationNotification)
      .map((item) => item.id) ?? []

    notificationsStore?.removeNotifications(ids)
  }
}

function notificationIcon(tone: CommunicationActionNotificationTone): string {
  switch (tone) {
    case 'success':
      return 'tabler:check'
    case 'warning':
      return 'tabler:alert-triangle'
    case 'danger':
      return 'tabler:alert-circle'
    default:
      return 'tabler:info-circle'
  }
}

function notificationToastVariant(
  tone: CommunicationActionNotificationTone
): 'info' | 'success' | 'warning' | 'error' {
  if (tone === 'danger') return 'error'

  return tone
}

function isTransientCommunicationFailure(notification: NotificationItem): boolean {
  return (
    isObsoleteCommunicationNotification(notification) ||
    (
      notification.sourceLabel === 'Mail' &&
      notification.title === 'Mail action failed' &&
      (
        notification.body?.includes('Failed to fetch') === true ||
        notification.body?.includes('Backend API is unavailable') === true
      )
    )
  )
}

function isObsoleteCommunicationNotification(notification: NotificationItem): boolean {
  return (
    notification.sourceLabel === 'Mail' &&
    notification.title === 'Mail action failed' &&
    notification.body?.includes('invalid raw signal event type: signal.raw.ai.') === true
  )
}
