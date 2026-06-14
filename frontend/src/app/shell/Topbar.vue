<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useNavigationStore } from '../../shared/stores/navigation'
import { useNotificationsStore } from '../../shared/stores/notifications'
import { useThemeStore } from '../../shared/stores/theme'
import { useLayoutEditorStore } from '../../shared/stores/layoutEditor'
import { useI18n } from '../../platform/i18n'

const nav = useNavigationStore()
const notifications = useNotificationsStore()
const theme = useThemeStore()
const layoutEditor = useLayoutEditorStore()
const { t, setLocale, locale } = useI18n()

function toggleNotifications(): void {
  notifications.toggleNotificationsDrawer()
}

function toggleUserMenu(): void {
  nav.toggleUserMenu()
}

function toggleLayoutEditing(): void {
  if (layoutEditor.isLayoutEditing) {
    layoutEditor.saveLayoutSettings()
  } else {
    layoutEditor.startLayoutEditing()
  }
}

function toggleLocale(): void {
  setLocale(locale.value === 'ru' ? 'en' : 'ru')
}

function exitApplication(): void {
  // In Tauri this would call window.close()
  // In browser, just a placeholder
  window.close()
}
</script>

<template>
  <header class="topbar">
    <!-- Title Section -->
    <div class="topbar-title-section">
      <h1 class="topbar-title">{{ nav.activeView?.title ?? 'Hermes' }}</h1>
      <p class="topbar-subtitle">{{ nav.activeView?.subtitle ?? '' }}</p>
    </div>

    <!-- Actions -->
    <div class="topbar-actions">
      <!-- Notification Bell -->
      <button
        class="topbar-action-btn"
        @click="toggleNotifications"
        title="Notifications"
      >
        <Icon icon="tabler:bell" class="topbar-action-icon" />
        <span
          v-if="notifications.notificationCount > 0"
          class="topbar-badge"
        >
          {{ notifications.notificationCount > 9 ? '9+' : notifications.notificationCount }}
        </span>
      </button>

      <!-- User Menu -->
      <div class="topbar-user-menu-wrapper">
        <button
          class="topbar-action-btn topbar-user-btn"
          :class="{ active: nav.isUserMenuOpen }"
          @click="toggleUserMenu"
          title="User menu"
        >
          <Icon icon="tabler:user-circle" class="topbar-action-icon" />
        </button>

        <!-- Dropdown -->
        <div v-if="nav.isUserMenuOpen" class="topbar-dropdown" @mouseleave="nav.closeUserMenu()">
          <!-- Layout Editing -->
          <button
            class="topbar-dropdown-item"
            :class="{ active: layoutEditor.isLayoutEditing }"
            @click="toggleLayoutEditing"
          >
            <Icon
              :icon="layoutEditor.isLayoutEditing ? 'tabler:layout-grid-add' : 'tabler:layout'"
              class="topbar-dropdown-icon"
            />
            <span>{{ layoutEditor.isLayoutEditing ? t('actions.save') || 'Save Layout' : t('actions.edit_layout') || 'Edit Layout' }}</span>
          </button>

          <div class="topbar-dropdown-separator" />

          <!-- Locale Switch -->
          <button class="topbar-dropdown-item" @click="toggleLocale">
            <Icon icon="tabler:language" class="topbar-dropdown-icon" />
            <span>{{ locale === 'ru' ? 'English' : 'Русский' }}</span>
          </button>

          <div class="topbar-dropdown-separator" />

          <!-- Exit -->
          <button class="topbar-dropdown-item topbar-dropdown-exit" @click="exitApplication">
            <Icon icon="tabler:logout" class="topbar-dropdown-icon" />
            <span>{{ t('actions.exit') || 'Exit' }}</span>
          </button>
        </div>
      </div>
    </div>
  </header>
</template>

<style scoped>
.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 3.5rem;
  padding: 0 1.25rem;
  background: var(--hh-panel-bg);
  border-bottom: 1px solid var(--hh-border);
  flex-shrink: 0;
}

.topbar-title-section {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  min-width: 0;
}

.topbar-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--hh-text-primary);
  margin: 0;
  line-height: 1.2;
}

.topbar-subtitle {
  font-size: 0.75rem;
  color: var(--hh-text-muted);
  margin: 0;
}

.topbar-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.topbar-action-btn {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.25rem;
  height: 2.25rem;
  border-radius: 0.375rem;
  border: none;
  background: transparent;
  color: var(--hh-text-secondary);
  cursor: pointer;
  transition: all 150ms ease;
}

.topbar-action-btn:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.topbar-action-btn.active {
  background: var(--hh-active-bg);
  color: var(--hh-accent);
}

.topbar-action-icon {
  width: 1.25rem;
  height: 1.25rem;
}

.topbar-badge {
  position: absolute;
  top: 0.25rem;
  right: 0.25rem;
  min-width: 1rem;
  height: 1rem;
  padding: 0 0.25rem;
  font-size: 0.625rem;
  font-weight: 700;
  line-height: 1rem;
  text-align: center;
  border-radius: 0.5rem;
  background: var(--hh-accent);
  color: var(--hh-bg);
}

.topbar-user-menu-wrapper {
  position: relative;
}

.topbar-dropdown {
  position: absolute;
  right: 0;
  top: 100%;
  margin-top: 0.25rem;
  width: 200px;
  background: var(--hh-panel-bg);
  border: 1px solid var(--hh-border);
  border-radius: 0.5rem;
  padding: 0.25rem;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  z-index: 100;
}

.topbar-dropdown-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: none;
  border-radius: 0.25rem;
  background: transparent;
  color: var(--hh-text-secondary);
  cursor: pointer;
  font-size: 0.8125rem;
  text-align: left;
  transition: all 150ms ease;
}

.topbar-dropdown-item:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.topbar-dropdown-item.active {
  color: var(--hh-accent);
}

.topbar-dropdown-item.topbar-dropdown-exit:hover {
  color: var(--hh-danger, #ef4444);
}

.topbar-dropdown-icon {
  width: 1.125rem;
  height: 1.125rem;
  flex-shrink: 0;
}

.topbar-dropdown-separator {
  height: 1px;
  background: var(--hh-border);
  margin: 0.25rem 0.5rem;
}
</style>
