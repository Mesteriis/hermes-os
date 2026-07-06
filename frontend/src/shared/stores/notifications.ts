import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type NotificationItem = {
  id: string
  title: string
  body?: string
  icon: string
  tone?: 'info' | 'success' | 'warning' | 'danger'
  sourceLabel?: string
  time: Date
  targetView?: string
  targetId?: string
  dedupeKey?: string
}

export const useNotificationsStore = defineStore('notifications', () => {
  const isNotificationsDrawerOpen = ref(false)
  const dismissedNotificationIds = ref<Set<string>>(new Set())
  const expandedNotificationIds = ref<Set<string>>(new Set())
  const rawNotificationItems = ref<NotificationItem[]>([])
  const pendingNotificationTarget = ref<NotificationItem | null>(null)

  const notificationItems = computed<NotificationItem[]>(() => {
    return rawNotificationItems.value
      .filter((item) => !dismissedNotificationIds.value.has(item.id))
      .filter((item) => !isSuppressedNotification(item))
      .sort((a, b) => b.time.getTime() - a.time.getTime())
      .slice(0, 12)
  })

  const notificationCount = computed<number>(() => {
    return notificationItems.value.length
  })

  function toggleNotificationsDrawer(): void {
    isNotificationsDrawerOpen.value = !isNotificationsDrawerOpen.value
  }

  function closeNotificationsDrawer(): void {
    isNotificationsDrawerOpen.value = false
  }

  function dismissNotification(notificationId: string): void {
    dismissedNotificationIds.value = new Set([...dismissedNotificationIds.value, notificationId])
  }

  function toggleNotificationExpanded(notificationId: string): void {
    const newSet = new Set(expandedNotificationIds.value)
    if (newSet.has(notificationId)) {
      newSet.delete(notificationId)
    } else {
      newSet.add(notificationId)
    }
    expandedNotificationIds.value = newSet
  }

  function openNotificationTarget(notification: NotificationItem): void {
    pendingNotificationTarget.value = notification
    closeNotificationsDrawer()
  }

  function consumePendingNotificationTarget(): NotificationItem | null {
    const target = pendingNotificationTarget.value
    pendingNotificationTarget.value = null
    return target
  }

  function addNotification(notification: NotificationItem): void {
    if (isSuppressedNotification(notification)) return

    const existingItems = notification.dedupeKey
      ? rawNotificationItems.value.filter((item) => item.dedupeKey !== notification.dedupeKey)
      : rawNotificationItems.value

    rawNotificationItems.value = [notification, ...existingItems].slice(0, 64)
  }

  function removeNotifications(notificationIds: readonly string[]): void {
    if (notificationIds.length === 0) return

    const ids = new Set(notificationIds)
    rawNotificationItems.value = rawNotificationItems.value.filter((item) => !ids.has(item.id))
    dismissedNotificationIds.value = new Set(
      [...dismissedNotificationIds.value].filter((id) => !ids.has(id))
    )
    expandedNotificationIds.value = new Set(
      [...expandedNotificationIds.value].filter((id) => !ids.has(id))
    )
  }

  function clearNotifications(): void {
    rawNotificationItems.value = []
    dismissedNotificationIds.value = new Set()
    expandedNotificationIds.value = new Set()
    pendingNotificationTarget.value = null
  }

  return {
    isNotificationsDrawerOpen,
    dismissedNotificationIds,
    expandedNotificationIds,
    rawNotificationItems,
    pendingNotificationTarget,
    notificationItems,
    notificationCount,
    toggleNotificationsDrawer,
    closeNotificationsDrawer,
    dismissNotification,
    toggleNotificationExpanded,
    openNotificationTarget,
    consumePendingNotificationTarget,
    addNotification,
    removeNotifications,
    clearNotifications
  }
})

function isSuppressedNotification(notification: NotificationItem): boolean {
  return (
    isLegacyAiSignalHubFailure(notification) ||
    isTransientMailNetworkFailure(notification)
  )
}

function isLegacyAiSignalHubFailure(notification: NotificationItem): boolean {
  return (
    notification.sourceLabel === 'Mail' &&
    notification.title === 'Mail action failed' &&
    notification.body?.includes('invalid raw signal event type: signal.raw.ai.') === true
  )
}

function isTransientMailNetworkFailure(notification: NotificationItem): boolean {
  return (
    notification.sourceLabel === 'Mail' &&
    notification.title === 'Mail action failed' &&
    (
      notification.body?.includes('Failed to fetch') === true ||
      notification.body?.includes('Backend API is unavailable') === true
    )
  )
}
