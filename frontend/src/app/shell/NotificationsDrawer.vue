<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useNotificationsDrawerSurface } from '../queries/useNotificationsDrawerSurface'

const {
  canExpandNotification,
  closeDrawer,
  formatTime,
  handleOpenTarget,
  isExpanded,
  notificationBodyPreview,
  notifications,
  t
} = useNotificationsDrawerSurface()
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
                  {{ notificationBodyPreview(notification) }}
                </div>
              </div>
            </button>
            <div class="notification-actions">
              <button
                v-if="canExpandNotification(notification)"
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

