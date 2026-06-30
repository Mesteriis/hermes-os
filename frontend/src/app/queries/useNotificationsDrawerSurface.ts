import { useI18n } from '../../platform/i18n'
import { useNavigationStore } from '../../shared/stores/navigation'
import { useNotificationsStore, type NotificationItem } from '../../shared/stores/notifications'

const MAX_NOTIFICATION_BODY_PREVIEW_LENGTH = 120

export function useNotificationsDrawerSurface() {
  const notifications = useNotificationsStore()
  const nav = useNavigationStore()
  const { t } = useI18n()

  function closeDrawer(): void {
    notifications.closeNotificationsDrawer()
  }

  function handleOpenTarget(notificationId: string): void {
    const notification = notifications.notificationItems.find((item) => item.id === notificationId)
    if (!notification) return
    notifications.openNotificationTarget(notification)
    if (notification.targetView) {
      nav.navigateTo(notification.targetView as Parameters<typeof nav.navigateTo>[0])
    }
  }

  function isExpanded(notificationId: string): boolean {
    return notifications.expandedNotificationIds.has(notificationId)
  }

  function formatTime(date: Date): string {
    return date.toLocaleString()
  }

  function canExpandNotification(notification: NotificationItem): boolean {
    return Boolean(notification.body && notification.body.length > MAX_NOTIFICATION_BODY_PREVIEW_LENGTH)
  }

  function notificationBodyPreview(notification: NotificationItem): string {
    const body = notification.body ?? ''
    if (!body) return ''
    if (body.length <= MAX_NOTIFICATION_BODY_PREVIEW_LENGTH || isExpanded(notification.id)) return body
    return `${body.slice(0, MAX_NOTIFICATION_BODY_PREVIEW_LENGTH)}…`
  }

  return {
    canExpandNotification,
    closeDrawer,
    formatTime,
    handleOpenTarget,
    isExpanded,
    nav,
    notificationBodyPreview,
    notifications,
    t
  }
}
