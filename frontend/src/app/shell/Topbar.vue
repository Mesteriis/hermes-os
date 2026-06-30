<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useTopbarSurface } from '../queries/useTopbarSurface'

const {
  exitApplication,
  locale,
  localeToggleLabel,
  nav,
  notificationBadgeLabel,
  notifications,
  realtimeStatus,
  realtimeStatusIcon,
  t,
  toggleLocale,
  toggleMenu,
  toggleNotifications
} = useTopbarSurface()
</script>

<template>
  <header class="topbar">
    <div class="topbar-slot-shell">
      <div id="hermes-topbar-slot" class="topbar-slot" />
      <div class="topbar-slot-fallback">
        <h1 class="topbar-title">{{ nav.activeView?.title ?? 'Hermes' }}</h1>
        <p class="topbar-subtitle">{{ nav.activeView?.subtitle ?? '' }}</p>
      </div>
    </div>

    <div class="topbar-actions">
      <div
        class="topbar-realtime-status"
        :class="realtimeStatus.realtimeStatusTone"
        :title="realtimeStatus.realtimeStatusDetail"
        :aria-label="realtimeStatus.realtimeStatusDetail"
      >
        <span class="topbar-realtime-dot" />
        <Icon :icon="realtimeStatusIcon" class="topbar-realtime-icon" aria-hidden="true" />
        <span class="topbar-realtime-label">{{ realtimeStatus.realtimeStatusLabel }}</span>
      </div>

      <button
        class="topbar-action-btn"
        @click="toggleNotifications"
        title="Notifications"
        aria-label="Notifications"
      >
        <Icon icon="tabler:bell" class="topbar-action-icon" />
        <span
          v-if="notifications.notificationCount > 0"
          class="topbar-badge"
        >
          {{ notificationBadgeLabel }}
        </span>
      </button>

      <div class="topbar-menu-wrapper">
        <button
          class="topbar-action-btn topbar-menu-btn"
          :class="{ active: nav.isUserMenuOpen }"
          @click="toggleMenu"
          title="Menu"
          aria-label="Menu"
        >
          <Icon icon="tabler:menu-2" class="topbar-action-icon" />
        </button>

        <div v-if="nav.isUserMenuOpen" class="topbar-dropdown" @mouseleave="nav.closeUserMenu()">
          <button class="topbar-dropdown-item" @click="toggleLocale">
            <Icon icon="tabler:language" class="topbar-dropdown-icon" />
            <span>{{ localeToggleLabel }}</span>
          </button>

          <div class="topbar-dropdown-separator" />

          <button class="topbar-dropdown-item topbar-dropdown-exit" @click="exitApplication">
            <Icon icon="tabler:logout" class="topbar-dropdown-icon" />
            <span>{{ t('actions.exit') || 'Exit' }}</span>
          </button>
        </div>
      </div>
    </div>
  </header>
</template>

