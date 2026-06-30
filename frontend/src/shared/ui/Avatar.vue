<script setup lang="ts">
import { AvatarRoot, AvatarImage, AvatarFallback } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  src?: string
  alt?: string
  fallback?: string
  size?: 'sm' | 'md' | 'lg' | 'xl'
  class?: string
}>(), {
  alt: 'avatar',
  size: 'md'
})

const rootClasses = computed(() => [
  'hermes-avatar-root',
  `hermes-avatar--${props.size}`,
  props.class
])

const fallbackText = computed(() => {
  if (props.fallback) return props.fallback.slice(0, 2).toUpperCase()
  if (props.alt && props.alt !== 'avatar') return props.alt.slice(0, 2).toUpperCase()
  return '?'
})
</script>

<template>
  <AvatarRoot :class="rootClasses">
    <AvatarImage v-if="src" :src="src" :alt="alt" class="hermes-avatar-image" />
    <AvatarFallback class="hermes-avatar-fallback" :delay-ms="src ? 300 : 0">
      {{ fallbackText }}
    </AvatarFallback>
  </AvatarRoot>
</template>

