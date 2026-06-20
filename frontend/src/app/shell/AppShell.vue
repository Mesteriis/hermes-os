<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { RouterView } from 'vue-router'
import { useRoute } from 'vue-router'
import { useNavigationStore } from '../../shared/stores/navigation'
import { useThemeStore } from '../../shared/stores/theme'
import Sidebar from './Sidebar.vue'
import Topbar from './Topbar.vue'
import NotificationsDrawer from './NotificationsDrawer.vue'

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
</script>

<template>
  <div
    class="viewport-guard"
    :class="[theme.shellThemeClass, nav.shellViewClass]"
  >
    <div
      class="desktop-shell"
      :class="{
        'sidebar-rail': nav.isSidebarRail
      }"
    >
      <!-- Sidebar -->
      <Sidebar />

      <!-- Workspace -->
      <div class="workspace">
        <Topbar />
        <NotificationsDrawer />
        <main class="workspace-content">
          <RouterView />
        </main>
      </div>
    </div>
  </div>
</template>

<style scoped>
.viewport-guard {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.desktop-shell {
  position: fixed;
  inset: 0;
  display: grid;
  grid-template-columns: var(--hh-shell-sidebar-width) minmax(var(--hh-shell-content-min-width), 1fr);
  gap: 16px;
  width: 100vw;
  max-width: 100vw;
  height: 100dvh;
  min-height: 0;
  overflow: hidden;
  padding: 0 var(--hh-shell-right-inset) var(--hh-shell-bottom-inset) 0;
  transition: grid-template-columns 280ms cubic-bezier(0.22, 1, 0.36, 1);
}

.desktop-shell.sidebar-rail {
  grid-template-columns: var(--hh-shell-sidebar-width-rail) minmax(var(--hh-shell-content-min-width), 1fr);
}

.workspace {
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: var(--hh-shell-workspace-gap);
  height: 100%;
  min-width: 0;
  overflow: hidden;
  padding-bottom: var(--hh-shell-topbar-offset);
}

.workspace-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 0;
}
</style>
