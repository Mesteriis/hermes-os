<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { DataListItem } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  items: DataListItem[]
  label?: string
  emptyText?: string
  density?: 'compact' | 'regular'
  class?: string
}>(), {
  emptyText: 'No items',
  density: 'regular'
})

const classes = computed(() => [
  'hermes-list',
  `hermes-list--${props.density}`,
  props.class
])
</script>

<template>
  <ul :class="classes" :aria-label="label">
    <li v-if="items.length === 0" class="hermes-list-empty">{{ emptyText }}</li>
    <template v-else>
      <li
        v-for="item in items"
        :key="item.id"
        :class="['hermes-list-item', `hermes-list-item--${item.tone ?? 'neutral'}`]"
      >
        <Icon v-if="item.icon" :icon="item.icon" size="1rem" class="hermes-list-icon" />
        <div class="hermes-list-copy">
          <strong class="hermes-list-label">{{ item.label }}</strong>
          <span v-if="item.description" class="hermes-list-description">{{ item.description }}</span>
        </div>
        <span v-if="item.meta" class="hermes-list-meta">{{ item.meta }}</span>
      </li>
    </template>
  </ul>
</template>
