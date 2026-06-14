<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useNavigationStore } from '../../shared/stores/navigation'
import { useSidebarStore } from '../../shared/stores/sidebar'
import type { ResolvedSidebarRootEntry } from '../../shared/stores/sidebar'

const nav = useNavigationStore()
const sidebar = useSidebarStore()

function handleSelectItem(viewId: string): void {
  nav.navigateTo(viewId as any)
}

function handleToggleGroup(groupId: string): void {
  nav.toggleSidebarGroup(groupId)
}

function handleToggleRail(): void {
  nav.toggleSidebarRail()
}

function handleSettings(): void {
  nav.navigateTo('settings')
}

function isItemActive(itemViewId: string): boolean {
  if (nav.currentView === 'communications') {
    return itemViewId === 'communications'
  }
  return nav.currentView === itemViewId
}

function isCommunicationItemActive(sectionId?: string): boolean {
  if (!sectionId || nav.currentView !== 'communications') return false
  return nav.activeCommunicationSection === sectionId
}
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
        <span class="sidebar-logo-mark">H</span>
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
              :class="{ 'rail-active': nav.activeSidebarRailGroupId === entry.group.id }"
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
      <button class="sidebar-nav-item" @click="handleSettings">
        <Icon icon="tabler:settings" class="sidebar-nav-icon" />
        <span v-if="!nav.isSidebarRail" class="sidebar-nav-label">Settings</span>
      </button>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--hh-panel-bg);
  border-right: 1px solid var(--hh-border);
  width: var(--hh-shell-sidebar-width);
  transition: width 280ms cubic-bezier(0.22, 1, 0.36, 1);
  overflow: hidden;
  z-index: 10;
}

.sidebar.sidebar-rail {
  width: var(--hh-shell-sidebar-width-rail);
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 1rem;
  cursor: pointer;
  border-bottom: 1px solid var(--hh-border);
  flex-shrink: 0;
}

.sidebar-logo {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--hh-accent);
  border-radius: 0.5rem;
  flex-shrink: 0;
}

.sidebar-logo-mark {
  font-size: 1rem;
  font-weight: 700;
  color: var(--hh-bg);
}

.sidebar-brand-text {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  white-space: nowrap;
}

.sidebar-brand-name {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--hh-text-primary);
}

.sidebar-brand-subtitle {
  font-size: 0.75rem;
  color: var(--hh-text-muted);
}

.sidebar-nav {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.sidebar-nav-item-wrapper {
  /* container for single items */
}

.sidebar-nav-group-wrapper {
  /* container for groups */
}

.sidebar-nav-group-header {
  /* group header row */
}

.sidebar-nav-group-toggle {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  border-radius: 0.375rem;
  border: none;
  background: transparent;
  color: var(--hh-text-primary);
  cursor: pointer;
  font-size: 0.8125rem;
  font-weight: 500;
  transition: background 150ms ease;
}

.sidebar-nav-group-toggle:hover {
  background: var(--hh-hover-bg);
}

.sidebar-nav-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  border-radius: 0.375rem;
  border: none;
  background: transparent;
  color: var(--hh-text-secondary);
  cursor: pointer;
  font-size: 0.8125rem;
  transition: all 150ms ease;
  text-align: left;
}

.sidebar-nav-item:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.sidebar-nav-item.active {
  background: var(--hh-active-bg);
  color: var(--hh-accent);
}

.sidebar-nav-icon {
  width: 1.25rem;
  height: 1.25rem;
  flex-shrink: 0;
}

.sidebar-nav-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-nav-chevron {
  margin-left: auto;
  width: 1rem;
  height: 1rem;
  transition: transform 200ms ease;
}

.sidebar-nav-chevron.expanded {
  transform: rotate(180deg);
}

.sidebar-nav-subnav {
  max-height: 0;
  overflow: hidden;
  transition: max-height 250ms ease;
}

.sidebar-nav-subnav.open {
  max-height: 500px;
}

.sidebar-nav-subitem {
  padding-left: 2.75rem;
  font-size: 0.75rem;
}

.sidebar-rail-group {
  position: relative;
}

.sidebar-rail-dropdown {
  position: absolute;
  left: 100%;
  top: 0;
  width: 200px;
  background: var(--hh-panel-bg);
  border: 1px solid var(--hh-border);
  border-radius: 0.5rem;
  padding: 0.25rem;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  z-index: 100;
  margin-left: 0.25rem;
}

.sidebar-rail-dropdown-item {
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

.sidebar-rail-dropdown-item:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.sidebar-rail-dropdown-item.active {
  color: var(--hh-accent);
}

.sidebar-nav-item.rail-active {
  background: var(--hh-active-bg);
  color: var(--hh-accent);
}

.sidebar-footer {
  border-top: 1px solid var(--hh-border);
  padding: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  flex-shrink: 0;
}
</style>
