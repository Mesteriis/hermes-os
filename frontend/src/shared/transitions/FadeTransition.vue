<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  duration?: number
  mode?: 'in-out' | 'out-in'
  appear?: boolean
}>(), {
  duration: 200,
  mode: 'out-in',
  appear: false
})

const cssDuration = computed(() => `${props.duration}ms`)
</script>

<template>
  <Transition
    :name="'hermes-fade'"
    :mode="mode"
    :appear="appear"
  >
    <slot />
  </Transition>
</template>

<style scoped>
.hermes-fade-enter-active,
.hermes-fade-leave-active {
  transition: opacity v-bind(cssDuration) ease;
}

.hermes-fade-enter-from,
.hermes-fade-leave-to {
  opacity: 0;
}
</style>
