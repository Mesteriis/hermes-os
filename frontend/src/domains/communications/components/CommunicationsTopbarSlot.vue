<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'

const props = defineProps<{
  searchQuery: string
  isSyncBusy: boolean
}>()

const emit = defineEmits<{
  'update:searchQuery': [query: string]
  search: []
  openAccountSetup: []
  compose: []
  syncNow: []
}>()

function updateSearchQuery(event: Event) {
  emit('update:searchQuery', (event.target as HTMLInputElement).value)
}
</script>

<template>
  <div class="communications-topbar-slot">
    <div class="communications-topbar-main">
      <h1 class="communications-topbar-title">Mail</h1>
      <label class="communications-topbar-search" aria-label="Search messages">
        <Icon icon="tabler:search" class="search-icon" />
        <input
          type="text"
          placeholder="Search messages..."
          :value="searchQuery"
          @input="updateSearchQuery"
          @keyup.enter="emit('search')"
        />
      </label>

      <div class="communications-topbar-actions">
        <button
          type="button"
          class="communications-topbar-icon-btn"
          title="Add mail account"
          aria-label="Add mail account"
          @click="emit('openAccountSetup')"
        >
          <Icon icon="tabler:mail-plus" />
        </button>
        <button
          type="button"
          class="communications-topbar-icon-btn"
          title="Compose"
          aria-label="Compose"
          @click="emit('compose')"
        >
          <Icon icon="tabler:edit" />
        </button>
        <button
          type="button"
          class="communications-topbar-icon-btn"
          :disabled="isSyncBusy"
          title="Refresh"
          aria-label="Refresh"
          @click="emit('syncNow')"
        >
          <Icon icon="tabler:refresh" :class="isSyncBusy ? 'spin-icon' : ''" />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.communications-topbar-slot {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  width: 100%;
  min-width: 0;
  height: 100%;
  padding: 0;
}

.communications-topbar-main {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex: 1;
  min-width: 0;
}

.communications-topbar-title {
  margin: 0;
  color: var(--hh-text-primary, #1f2937);
  font-size: 1rem;
  font-weight: 760;
  line-height: 1;
  white-space: nowrap;
}

.communications-topbar-search {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  flex: 1;
  height: 2rem;
  min-width: 12rem;
  max-width: 32rem;
  padding: 0 0.625rem;
  border: 1px solid var(--hh-border-subtle, #e5e7eb);
  border-radius: var(--hh-radius-control, 0.375rem);
  background: rgba(2, 12, 16, calc(var(--hh-panel-alpha, 0.7) * 0.72));
  color: var(--hh-text-primary, #1f2937);
}

.search-icon {
  flex-shrink: 0;
  width: 14px;
  height: 14px;
  color: var(--hh-text-tertiary, #9ca3af);
}

.communications-topbar-search input {
  flex: 1;
  min-width: 0;
  border: none;
  outline: none;
  background: transparent;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
}

.communications-topbar-search input::placeholder {
  color: var(--hh-text-tertiary, #9ca3af);
}

.communications-topbar-actions {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  gap: 0.25rem;
}

.communications-topbar-icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border: 1px solid transparent;
  border-radius: var(--hh-radius-control, 0.375rem);
  background: transparent;
  color: var(--hh-text-secondary, #6b7280);
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease, border-color 150ms ease, opacity 150ms ease;
}

.communications-topbar-icon-btn:hover:not(:disabled) {
  border-color: var(--hh-border-subtle, #e5e7eb);
  background: var(--hh-hover-bg, rgba(255, 255, 255, 0.06));
  color: var(--hh-text-primary, #1f2937);
}

.communications-topbar-icon-btn:focus-visible {
  outline: 2px solid var(--hh-focus-ring);
  outline-offset: 2px;
}

.communications-topbar-icon-btn:disabled {
  cursor: not-allowed;
  opacity: 0.48;
}

.communications-topbar-icon-btn :deep(svg) {
  width: 1.125rem;
  height: 1.125rem;
}

.spin-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
