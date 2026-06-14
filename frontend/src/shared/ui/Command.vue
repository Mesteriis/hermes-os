<script setup lang="ts">
import { DialogRoot, DialogPortal, DialogOverlay, DialogContent } from 'reka-ui'
import { ref, computed, watch, nextTick } from 'vue'
import Icon from './Icon.vue'

export interface CommandGroup {
  label: string
  items: CommandItem[]
}

export interface CommandItem {
  id: string
  label: string
  description?: string
  icon?: string
  keywords?: string[]
  onSelect?: () => void
}

const props = withDefaults(defineProps<{
  open?: boolean
  groups?: CommandGroup[]
  placeholder?: string
  emptyMessage?: string
  class?: string
  contentClass?: string
}>(), {
  placeholder: 'Поиск...',
  emptyMessage: 'Ничего не найдено'
})

const emit = defineEmits<{
  'update:open': [value: boolean]
  'select': [item: CommandItem]
}>()

const query = ref('')
const inputRef = ref<HTMLInputElement | null>(null)
const selectedIndex = ref(0)

const flatItems = computed(() => {
  return (props.groups || []).flatMap((g) => g.items)
})

const filteredGroups = computed(() => {
  const q = query.value.toLowerCase().trim()
  if (!q) return props.groups || []

  return (props.groups || [])
    .map((group) => ({
      ...group,
      items: group.items.filter((item) => {
        const labelMatch = item.label.toLowerCase().includes(q)
        const descMatch = item.description?.toLowerCase().includes(q)
        const keywordMatch = item.keywords?.some((k) => k.toLowerCase().includes(q))
        return labelMatch || descMatch || keywordMatch
      })
    }))
    .filter((g) => g.items.length > 0)
})

const filteredFlatItems = computed(() => {
  return filteredGroups.value.flatMap((g) => g.items)
})

watch(() => props.open, (isOpen) => {
  if (isOpen) {
    query.value = ''
    selectedIndex.value = 0
    nextTick(() => inputRef.value?.focus())
  }
})

function handleKeyDown(event: KeyboardEvent): void {
  const items = filteredFlatItems.value
  if (items.length === 0) return

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      selectedIndex.value = Math.min(selectedIndex.value + 1, items.length - 1)
      break
    case 'ArrowUp':
      event.preventDefault()
      selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
      break
    case 'Enter':
      event.preventDefault()
      const selected = items[selectedIndex.value]
      if (selected) {
        selected.onSelect?.()
        emit('select', selected)
        emit('update:open', false)
      }
      break
  }
}

function selectItem(item: CommandItem): void {
  item.onSelect?.()
  emit('select', item)
  emit('update:open', false)
}

const contentClasses = computed(() => [
  'hermes-command-content',
  props.contentClass
])
</script>

<template>
  <DialogRoot :open="open" @update:open="(val) => emit('update:open', val)">
    <DialogPortal>
      <DialogOverlay class="hermes-command-overlay" @pointerdown="emit('update:open', false)">
        <DialogContent :class="contentClasses" @keydown="handleKeyDown" @open-auto-focus="(e: Event) => e.preventDefault()">
          <div class="hermes-command-input-wrapper">
            <Icon icon="tabler:search" size="1.125rem" class="hermes-command-search-icon" />
            <input
              ref="inputRef"
              v-model="query"
              class="hermes-command-input"
              :placeholder="placeholder"
              @keydown.stop="handleKeyDown"
            />
            <kbd class="hermes-command-kbd">ESC</kbd>
          </div>

          <div class="hermes-command-list">
            <template v-if="filteredGroups.length > 0">
              <div v-for="(group, gi) in filteredGroups" :key="gi" class="hermes-command-group">
                <div class="hermes-command-group-label">{{ group.label }}</div>
                <button
                  v-for="(item, ii) in group.items"
                  :key="item.id"
                  class="hermes-command-item"
                  :class="{ 'hermes-command-item--selected': flatItems.indexOf(item) === selectedIndex }"
                  @click="selectItem(item)"
                  @mouseenter="selectedIndex = flatItems.indexOf(item)"
                >
                  <Icon v-if="item.icon" :icon="item.icon" size="1.125rem" class="hermes-command-item-icon" />
                  <div class="hermes-command-item-text">
                    <span class="hermes-command-item-label">{{ item.label }}</span>
                    <span v-if="item.description" class="hermes-command-item-desc">{{ item.description }}</span>
                  </div>
                </button>
              </div>
            </template>
            <div v-else-if="query" class="hermes-command-empty">
              <Icon icon="tabler:search-off" size="1.5rem" />
              <span>{{ emptyMessage }}</span>
            </div>
          </div>
        </DialogContent>
      </DialogOverlay>
    </DialogPortal>
  </DialogRoot>
</template>

<style scoped>
.hermes-command-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  z-index: 100;
  padding-top: 12vh;
  animation: command-overlay-in 150ms ease;
}

.hermes-command-content {
  width: 90vw;
  max-width: 560px;
  max-height: 60vh;
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-lg);
  box-shadow: var(--hh-shadow-modal);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  animation: command-content-in 150ms ease;
}

.hermes-command-input-wrapper {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.875rem 1rem;
  border-bottom: 1px solid var(--hh-border);
  flex-shrink: 0;
}

.hermes-command-search-icon {
  flex-shrink: 0;
  color: var(--hh-text-muted);
}

.hermes-command-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: 0.875rem;
  color: var(--hh-text-primary);
  font-family: inherit;
  line-height: 1.5;
}

.hermes-command-input::placeholder {
  color: var(--hh-text-muted);
}

.hermes-command-kbd {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.125rem 0.375rem;
  font-size: 0.625rem;
  font-weight: 500;
  color: var(--hh-text-muted);
  background: var(--hh-hover-bg);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-xs);
  font-family: inherit;
  line-height: 1.4;
}

.hermes-command-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
}

.hermes-command-group {
  margin-bottom: 0.25rem;
}

.hermes-command-group-label {
  padding: 0.375rem 0.5rem;
  font-size: 0.6875rem;
  font-weight: 600;
  color: var(--hh-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.hermes-command-item {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  width: 100%;
  padding: 0.5rem 0.625rem;
  border-radius: var(--hh-radius-sm);
  background: transparent;
  border: none;
  cursor: pointer;
  text-align: left;
  font-family: inherit;
  transition: background 100ms ease;
}

.hermes-command-item:hover,
.hermes-command-item--selected {
  background: var(--hh-hover-bg);
}

.hermes-command-item:focus-visible {
  outline: 2px solid var(--hh-focus-ring);
  outline-offset: -2px;
}

.hermes-command-item-icon {
  flex-shrink: 0;
  color: var(--hh-text-secondary);
}

.hermes-command-item-text {
  flex: 1;
  min-width: 0;
}

.hermes-command-item-label {
  display: block;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-primary);
  line-height: 1.4;
}

.hermes-command-item-desc {
  display: block;
  font-size: 0.6875rem;
  color: var(--hh-text-muted);
  line-height: 1.3;
  margin-top: 0.0625rem;
}

.hermes-command-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  padding: 2rem 1rem;
  color: var(--hh-text-muted);
  font-size: 0.8125rem;
}

@keyframes command-overlay-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes command-content-in {
  from {
    opacity: 0;
    transform: translateY(-1rem) scale(0.97);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}
</style>
