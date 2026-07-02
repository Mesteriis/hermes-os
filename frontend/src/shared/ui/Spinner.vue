<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  size?: 'sm' | 'md' | 'lg'
  label?: string
  decorative?: boolean
  class?: string
}>(), {
  size: 'md',
  label: 'Loading',
  decorative: false
})

const classes = computed(() => [
  'hermes-spinner',
  `hermes-spinner--${props.size}`,
  props.class
])

const role = computed(() => props.decorative ? undefined : 'status')
const ariaLabel = computed(() => props.decorative ? undefined : props.label)
</script>

<template>
  <span :class="classes" :role="role" :aria-label="ariaLabel">
    <span class="hermes-spinner-mark" aria-hidden="true" />
    <span v-if="!decorative && label" class="hermes-sr-only">{{ label }}</span>
  </span>
</template>
