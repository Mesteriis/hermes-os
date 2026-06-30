<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
  width?: string
  height?: string
  rounded?: boolean
}>(), {
  width: '100%',
  height: '1rem',
  rounded: false
})

const classes = computed(() => [
  'hermes-skeleton',
  { 'hermes-skeleton--rounded': props.rounded },
  props.class
])
</script>

<template>
  <div :class="classes" />
</template>

