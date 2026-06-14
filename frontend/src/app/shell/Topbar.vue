<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useNavigationStore } from '../../shared/stores/navigation'
import { useNotificationsStore } from '../../shared/stores/notifications'
import { useI18n } from '../../platform/i18n'

const nav = useNavigationStore()
const notifications = useNotificationsStore()
const { t, setLocale, locale } = useI18n()

function toggleNotifications(): void {
  notifications.toggleNotificationsDrawer()
}

function toggleMenu(): void {
  nav.toggleUserMenu()
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
    <div class="topbar-slot-shell">
      <div id="hermes-topbar-slot" class="topbar-slot" />
      <div class="topbar-slot-fallback">
        <h1 class="topbar-title">{{ nav.activeView?.title ?? 'Hermes' }}</h1>
        <p class="topbar-subtitle">{{ nav.activeView?.subtitle ?? '' }}</p>
      </div>
    </div>

    <div class="topbar-actions">
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
          {{ notifications.notificationCount > 9 ? '9+' : notifications.notificationCount }}
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
            <span>{{ locale === 'ru' ? 'English' : 'Русский' }}</span>
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

<style scoped>
.topbar {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.75rem;
  height: 3.5rem;
  margin-top: var(--hh-shell-topbar-offset);
  padding: 0 0.875rem;
  background: rgba(5, 22, 25, var(--hh-panel-alpha));
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  box-shadow: var(--hh-shadow-panel);
  backdrop-filter: blur(var(--hh-panel-blur));
  flex-shrink: 0;
}

.topbar-slot-shell {
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 0;
  height: 100%;
}

.topbar-slot {
  display: flex;
  align-items: stretch;
  flex: 1;
  min-width: 0;
  height: 100%;
}

.topbar-slot:has(> *) + .topbar-slot-fallback {
  display: none;
}

.topbar-slot-fallback {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  justify-content: center;
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
  flex-shrink: 0;
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

.topbar-menu-wrapper {
  position: relative;
  flex-shrink: 0;
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
