<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  icon: string
  size?: number | string
  class?: string
}>(), {
  size: 20
})

const iconClass = computed(() => props.class || '')
</script>

<template>
  <Icon
    :icon="icon"
    :width="size"
    :height="size"
    :class="iconClass"
    aria-hidden="true"
  />
</template>
