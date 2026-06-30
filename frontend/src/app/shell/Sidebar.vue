<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useSidebarSurface } from '../queries/useSidebarSurface'

const {
  nav,
  sidebar,
  handleSelectItem,
  handleToggleGroup,
  handleToggleRail,
  handleSettings,
  isItemActive,
  isCommunicationItemActive
} = useSidebarSurface()
</script>

<template>
  <aside
    class="sidebar"
    :class="{
      'sidebar-rail': nav.isSidebarRail
    }"
  >
    <!-- Brand -->
    <div class="sidebar-brand" @click="handleSelectItem('home')">
      <div class="sidebar-logo">
        <img class="sidebar-logo-image" src="/assets/hermes-logo-mark.png" alt="" aria-hidden="true" />
      </div>
      <div v-if="!nav.isSidebarRail" class="sidebar-brand-text">
        <span class="sidebar-brand-name">Hermes</span>
        <span class="sidebar-brand-subtitle">Memory System</span>
      </div>
    </div>

    <!-- Navigation -->
    <nav class="sidebar-nav">
      <template v-for="entry in sidebar.sidebarRootEntries" :key="entry.rootId">
        <!-- Single Item -->
        <div v-if="entry.kind === 'item'" class="sidebar-nav-item-wrapper">
          <button
            class="sidebar-nav-item"
            :class="{ active: isItemActive(entry.item.itemId) }"
            @click="handleSelectItem(entry.item.itemId)"
          >
            <Icon :icon="entry.item.icon" class="sidebar-nav-icon" />
            <span v-if="!nav.isSidebarRail" class="sidebar-nav-label">
              {{ entry.item.label }}
            </span>
          </button>
        </div>

        <!-- Group -->
        <div v-else-if="entry.kind === 'group'" class="sidebar-nav-group-wrapper">
          <!-- Group Header -->
          <div v-if="!nav.isSidebarRail" class="sidebar-nav-group-header">
            <button
              class="sidebar-nav-group-toggle"
              :class="{ active: isItemActive(entry.group.id) }"
              @click="handleToggleGroup(entry.group.id)"
            >
              <Icon
                :icon="entry.group.icon"
                class="sidebar-nav-icon"
              />
              <span class="sidebar-nav-label">{{ entry.group.label }}</span>
              <Icon
                icon="tabler:chevron-down"
                class="sidebar-nav-chevron"
                :class="{ expanded: nav.expandedSidebarGroupIds.includes(entry.group.id) }"
              />
            </button>
          </div>

          <!-- Group Items (normal mode) -->
          <div
            v-if="!nav.isSidebarRail"
            class="sidebar-nav-subnav"
            :class="{ open: nav.expandedSidebarGroupIds.includes(entry.group.id) }"
          >
            <button
              v-for="item in entry.group.items"
              :key="item.itemId"
              class="sidebar-nav-item sidebar-nav-subitem"
              :class="{
                active: item.isCommunication
                  ? isCommunicationItemActive(item.sectionId)
                  : isItemActive(item.itemId)
              }"
              @click="item.isCommunication
                ? nav.navigateToCommunicationSection(item.sectionId!)
                : handleSelectItem(item.itemId)"
            >
              <Icon :icon="item.icon" class="sidebar-nav-icon" />
              <span class="sidebar-nav-label">{{ item.label }}</span>
            </button>
          </div>

          <!-- Rail mode: group shows as dropdown trigger -->
          <div v-else class="sidebar-rail-group">
            <button
              class="sidebar-nav-item"
              :class="{
                'rail-active': nav.activeSidebarRailGroupId === entry.group.id || isItemActive(entry.group.id)
              }"
              @click="nav.setActiveSidebarRailGroup(
                nav.activeSidebarRailGroupId === entry.group.id ? null : entry.group.id
              )"
            >
              <Icon :icon="entry.group.icon" class="sidebar-nav-icon" />
            </button>
            <div
              v-if="nav.activeSidebarRailGroupId === entry.group.id"
              class="sidebar-rail-dropdown"
            >
              <button
                v-for="item in entry.group.items"
                :key="item.itemId"
                class="sidebar-rail-dropdown-item"
                :class="{
                  active: item.isCommunication
                    ? isCommunicationItemActive(item.sectionId)
                    : isItemActive(item.itemId)
                }"
                @click="item.isCommunication
                  ? nav.navigateToCommunicationSection(item.sectionId!)
                  : handleSelectItem(item.itemId)"
              >
                <Icon :icon="item.icon" class="sidebar-nav-icon" />
                <span>{{ item.label }}</span>
              </button>
            </div>
          </div>
        </div>
      </template>
    </nav>

    <!-- Bottom section -->
    <div class="sidebar-footer">
      <button class="sidebar-nav-item" @click="handleToggleRail">
        <Icon
          :icon="nav.isSidebarRail ? 'tabler:layout-sidebar-right' : 'tabler:layout-sidebar'"
          class="sidebar-nav-icon"
        />
        <span v-if="!nav.isSidebarRail" class="sidebar-nav-label">
          {{ nav.isSidebarRail ? 'Expand' : 'Collapse' }}
        </span>
      </button>
      <button
        class="sidebar-nav-item"
        :class="{ active: isItemActive('settings') }"
        @click="handleSettings"
      >
        <Icon icon="tabler:settings" class="sidebar-nav-icon" />
        <span v-if="!nav.isSidebarRail" class="sidebar-nav-label">Settings</span>
      </button>
    </div>
  </aside>
</template>

