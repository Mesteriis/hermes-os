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

