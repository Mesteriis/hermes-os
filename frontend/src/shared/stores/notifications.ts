import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type NotificationItem = {
  id: string
  title: string
  body?: string
  icon: string
  time: Date
  targetView?: string
  targetId?: string
}

export const useNotificationsStore = defineStore('notifications', () => {
  const isNotificationsDrawerOpen = ref(false)
  const dismissedNotificationIds = ref<Set<string>>(new Set())
  const expandedNotificationIds = ref<Set<string>>(new Set())
  const rawNotificationItems = ref<NotificationItem[]>([])
  const pendingNotificationTarget = ref<NotificationItem | null>(null)

  // In a real implementation, notification items would come from SSE events
  // For now, using an empty array as the shell-ready state

  const notificationItems = computed<NotificationItem[]>(() => {
    return rawNotificationItems.value
      .filter((item) => !dismissedNotificationIds.value.has(item.id))
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
    rawNotificationItems.value = [notification, ...rawNotificationItems.value]
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
    addNotification
  }
})
