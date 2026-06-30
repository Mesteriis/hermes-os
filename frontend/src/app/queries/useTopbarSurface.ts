import { computed } from 'vue'
import { useI18n } from '../../platform/i18n'
import { useNavigationStore } from '../../shared/stores/navigation'
import { useNotificationsStore } from '../../shared/stores/notifications'
import { useRealtimeStatusStore } from '../../shared/stores/realtimeStatus'

export function useTopbarSurface() {
  const nav = useNavigationStore()
  const notifications = useNotificationsStore()
  const realtimeStatus = useRealtimeStatusStore()
  const { t, setLocale, locale } = useI18n()

  const realtimeStatusIcon = computed<string>(() => {
    if (realtimeStatus.realtimeStatusTone === 'success') return 'tabler:cloud-check'
    if (realtimeStatus.realtimeStatusTone === 'danger') return 'tabler:cloud-off'
    if (realtimeStatus.isRealtimeDegraded) return 'tabler:cloud-exclamation'
    return 'tabler:cloud-up'
  })

  const localeToggleLabel = computed(() => (locale.value === 'ru' ? 'English' : 'Русский'))
  const notificationBadgeLabel = computed(() => (
    notifications.notificationCount > 9 ? '9+' : String(notifications.notificationCount)
  ))

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
    window.close()
  }

  return {
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
  }
}
