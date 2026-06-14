<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useNotificationsStore } from '../../shared/stores/notifications'
import { useNavigationStore } from '../../shared/stores/navigation'
import { useI18n } from '../../platform/i18n'

const notifications = useNotificationsStore()
const nav = useNavigationStore()
const { t } = useI18n()

function closeDrawer(): void {
  notifications.closeNotificationsDrawer()
}

function handleOpenTarget(notificationId: string): void {
  const notification = notifications.notificationItems.find(n => n.id === notificationId)
  if (notification) {
    notifications.openNotificationTarget(notification)
    if (notification.targetView) {
      nav.navigateTo(notification.targetView as any)
    }
  }
}

function isExpanded(notificationId: string): boolean {
  return notifications.expandedNotificationIds.has(notificationId)
}

function formatTime(date: Date): string {
  return date.toLocaleString()
}
</script>

<template>
  <div>
    <!-- Backdrop -->
    <Transition name="drawer-backdrop">
      <div
        v-if="notifications.isNotificationsDrawerOpen"
        class="drawer-backdrop"
        @click="closeDrawer"
      />
    </Transition>

    <!-- Drawer -->
    <Transition name="drawer-slide">
      <aside
        v-if="notifications.isNotificationsDrawerOpen"
        class="notifications-drawer"
      >
        <!-- Header -->
        <div class="drawer-header">
          <div class="drawer-header-left">
            <Icon icon="tabler:bell" class="drawer-header-icon" />
            <span class="drawer-title">{{ t('notifications.title') || 'Notifications' }}</span>
            <span class="drawer-count">{{ notifications.notificationCount }}</span>
          </div>
          <button class="drawer-close-btn" @click="closeDrawer">
            <Icon icon="tabler:x" class="drawer-close-icon" />
          </button>
        </div>

        <!-- Empty State -->
        <div v-if="notifications.notificationItems.length === 0" class="drawer-empty">
          <Icon icon="tabler:bell-check" class="drawer-empty-icon" />
          <p class="drawer-empty-text">{{ t('notifications.empty') || 'All caught up!' }}</p>
        </div>

        <!-- Notification List -->
        <div v-else class="drawer-list">
          <div
            v-for="notification in notifications.notificationItems"
            :key="notification.id"
            class="notification-item"
          >
            <button
              class="notification-main"
              @click="handleOpenTarget(notification.id)"
            >
              <Icon :icon="notification.icon" class="notification-icon" />
              <div class="notification-content">
                <div class="notification-header">
                  <span class="notification-title">{{ notification.title }}</span>
                  <span class="notification-time">{{ formatTime(notification.time) }}</span>
                </div>
                <div
                  v-if="notification.body"
                  class="notification-body"
                  :class="{ expanded: isExpanded(notification.id) }"
                >
                  {{ notification.body.length > 120 && !isExpanded(notification.id)
                    ? notification.body.slice(0, 120) + '…'
                    : notification.body
                  }}
                </div>
              </div>
            </button>
            <div class="notification-actions">
              <button
                v-if="notification.body && notification.body.length > 120"
                class="notification-action-btn"
                @click="notifications.toggleNotificationExpanded(notification.id)"
              >
                <Icon
                  :icon="isExpanded(notification.id) ? 'tabler:chevron-up' : 'tabler:chevron-down'"
                  class="notification-action-icon"
                />
              </button>
              <button
                class="notification-action-btn"
                @click="notifications.dismissNotification(notification.id)"
              >
                <Icon icon="tabler:x" class="notification-action-icon" />
              </button>
            </div>
          </div>
        </div>
      </aside>
    </Transition>
  </div>
</template>

<style scoped>
.drawer-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  z-index: 40;
}

.notifications-drawer {
  position: fixed;
  top: 0;
  right: 0;
  width: 380px;
  height: 100vh;
  background: var(--hh-panel-bg);
  border-left: 1px solid var(--hh-border);
  z-index: 50;
  display: flex;
  flex-direction: column;
  box-shadow: -4px 0 20px rgba(0, 0, 0, 0.2);
}

.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--hh-border);
  flex-shrink: 0;
}

.drawer-header-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.drawer-header-icon {
  width: 1.25rem;
  height: 1.25rem;
  color: var(--hh-accent);
}

.drawer-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--hh-text-primary);
}

.drawer-count {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--hh-accent);
  background: var(--hh-active-bg);
  padding: 0.125rem 0.5rem;
  border-radius: 0.75rem;
}

.drawer-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border-radius: 0.375rem;
  border: none;
  background: transparent;
  color: var(--hh-text-secondary);
  cursor: pointer;
  transition: all 150ms ease;
}

.drawer-close-btn:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.drawer-close-icon {
  width: 1.25rem;
  height: 1.25rem;
}

.drawer-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  padding: 2rem;
}

.drawer-empty-icon {
  width: 3rem;
  height: 3rem;
  color: var(--hh-text-muted);
}

.drawer-empty-text {
  font-size: 0.875rem;
  color: var(--hh-text-muted);
  margin: 0;
}

.drawer-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
}

.notification-item {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  padding: 0.75rem;
  border-radius: 0.375rem;
  transition: background 150ms ease;
}

.notification-item:hover {
  background: var(--hh-hover-bg);
}

.notification-main {
  flex: 1;
  display: flex;
  gap: 0.75rem;
  min-width: 0;
  border: none;
  background: transparent;
  cursor: pointer;
  text-align: left;
  padding: 0;
}

.notification-icon {
  width: 1.25rem;
  height: 1.25rem;
  color: var(--hh-accent);
  flex-shrink: 0;
  margin-top: 0.125rem;
}

.notification-content {
  min-width: 0;
}

.notification-header {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
  margin-bottom: 0.25rem;
}

.notification-title {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-primary);
}

.notification-time {
  font-size: 0.6875rem;
  color: var(--hh-text-muted);
  white-space: nowrap;
}

.notification-body {
  font-size: 0.75rem;
  color: var(--hh-text-secondary);
  line-height: 1.4;
}

.notification-body.expanded {
  /* full text shown */
}

.notification-actions {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  flex-shrink: 0;
}

.notification-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  border-radius: 0.25rem;
  border: none;
  background: transparent;
  color: var(--hh-text-muted);
  cursor: pointer;
  transition: all 150ms ease;
}

.notification-action-btn:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.notification-action-icon {
  width: 0.875rem;
  height: 0.875rem;
}

/* Transitions */
.drawer-backdrop-enter-active,
.drawer-backdrop-leave-active {
  transition: opacity 200ms ease;
}
.drawer-backdrop-enter-from,
.drawer-backdrop-leave-to {
  opacity: 0;
}

.drawer-slide-enter-active,
.drawer-slide-leave-active {
  transition: transform 280ms cubic-bezier(0.22, 1, 0.36, 1);
}
.drawer-slide-enter-from,
.drawer-slide-leave-to {
  transform: translateX(100%);
}
</style>
