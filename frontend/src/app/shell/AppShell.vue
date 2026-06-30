<script setup lang="ts">
import { RouterView } from 'vue-router'
import { useAppShellSurface } from '../queries/useAppShellSurface'
import Sidebar from './Sidebar.vue'
import Topbar from './Topbar.vue'
import NotificationsDrawer from './NotificationsDrawer.vue'

const { nav, theme } = useAppShellSurface()
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

