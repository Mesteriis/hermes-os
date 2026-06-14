<script setup lang="ts">
import { AvatarRoot, AvatarImage, AvatarFallback } from 'reka-ui'
import { computed } from 'vue'
import type { CSSProperties } from 'vue'

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

const sizeMap: Record<string, string> = {
  sm: '1.5rem',
  md: '2rem',
  lg: '2.5rem',
  xl: '3rem'
}

const rootStyle = computed<CSSProperties>(() => ({
  width: sizeMap[props.size],
  height: sizeMap[props.size]
}))

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
  <AvatarRoot :class="rootClasses" :style="rootStyle">
    <AvatarImage v-if="src" :src="src" :alt="alt" class="hermes-avatar-image" />
    <AvatarFallback class="hermes-avatar-fallback" :delay-ms="src ? 300 : 0">
      {{ fallbackText }}
    </AvatarFallback>
  </AvatarRoot>
</template>

<style scoped>
.hermes-avatar-root {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  vertical-align: middle;
  border-radius: 9999px;
  background: var(--hh-hover-bg);
  border: 1px solid var(--hh-border);
  overflow: hidden;
  flex-shrink: 0;
  user-select: none;
}

.hermes-avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: inherit;
}

.hermes-avatar-fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  font-weight: 600;
  color: var(--hh-text-secondary);
  line-height: 1;
}

.hermes-avatar--sm .hermes-avatar-fallback {
  font-size: 0.625rem;
}

.hermes-avatar--md .hermes-avatar-fallback {
  font-size: 0.75rem;
}

.hermes-avatar--lg .hermes-avatar-fallback {
  font-size: 0.875rem;
}

.hermes-avatar--xl .hermes-avatar-fallback {
  font-size: 1rem;
}
</style>
