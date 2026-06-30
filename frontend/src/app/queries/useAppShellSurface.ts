import { onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useNavigationStore } from '../../shared/stores/navigation'
import { useThemeStore } from '../../shared/stores/theme'

export function useAppShellSurface() {
  const nav = useNavigationStore()
  const theme = useThemeStore()
  const route = useRoute()

  onMounted(() => {
    void theme.hydrateThemeSettings()
  })

  watch(
    () => [route.name, route.query.section] as const,
    ([name, section]) => {
      if (typeof name === 'string') {
        nav.syncFromRoute(name as Parameters<typeof nav.syncFromRoute>[0], section)
      }
    },
    { immediate: true }
  )

  return {
    nav,
    theme
  }
}
