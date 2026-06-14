<script setup lang="ts">
import { RouterView } from 'vue-router'
import { useNavigationStore } from '../../shared/stores/navigation'
import { useThemeStore } from '../../shared/stores/theme'
import { useLayoutEditorStore } from '../../shared/stores/layoutEditor'
import Sidebar from './Sidebar.vue'
import Topbar from './Topbar.vue'
import NotificationsDrawer from './NotificationsDrawer.vue'
import LayoutEditControls from './LayoutEditControls.vue'

const nav = useNavigationStore()
const theme = useThemeStore()
const layoutEditor = useLayoutEditorStore()
</script>

<template>
  <div
    class="viewport-guard"
    :class="[theme.shellThemeClass, nav.shellViewClass]"
  >
    <div
      class="desktop-shell"
      :class="{
        'sidebar-rail': nav.isSidebarRail,
        'layout-editing': layoutEditor.isLayoutEditing
      }"
    >
      <!-- Sidebar -->
      <Sidebar />

      <!-- Workspace -->
      <div class="workspace">
        <Topbar />
        <LayoutEditControls />
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
  display: grid;
  grid-template-columns: var(--hh-shell-sidebar-width) 1fr;
  width: 100%;
  height: 100%;
  transition: grid-template-columns 280ms cubic-bezier(0.22, 1, 0.36, 1);
}

.desktop-shell.sidebar-rail {
  grid-template-columns: var(--hh-shell-sidebar-width-rail) 1fr;
}

.workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.workspace-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}
</style>
