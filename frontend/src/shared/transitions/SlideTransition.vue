<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  direction?: 'up' | 'down' | 'left' | 'right'
  duration?: number
  distance?: string
  mode?: 'in-out' | 'out-in'
  appear?: boolean
}>(), {
  direction: 'up',
  duration: 200,
  distance: '1rem',
  mode: 'out-in',
  appear: false
})

const cssDuration = computed(() => `${props.duration}ms`)
const cssDistance = computed(() => props.distance)

const nameClass = computed(() => `hermes-slide-${props.direction}`)
</script>

<template>
  <Transition
    :name="nameClass"
    :mode="mode"
    :appear="appear"
  >
    <slot />
  </Transition>
</template>

<style scoped>
/* Up */
.hermes-slide-up-enter-active,
.hermes-slide-up-leave-active {
  transition: all v-bind(cssDuration) cubic-bezier(0.16, 1, 0.3, 1);
}
.hermes-slide-up-enter-from {
  opacity: 0;
  transform: translateY(v-bind(cssDistance));
}
.hermes-slide-up-leave-to {
  opacity: 0;
  transform: translateY(calc(-1 * v-bind(cssDistance)));
}

/* Down */
.hermes-slide-down-enter-active,
.hermes-slide-down-leave-active {
  transition: all v-bind(cssDuration) cubic-bezier(0.16, 1, 0.3, 1);
}
.hermes-slide-down-enter-from {
  opacity: 0;
  transform: translateY(calc(-1 * v-bind(cssDistance)));
}
.hermes-slide-down-leave-to {
  opacity: 0;
  transform: translateY(v-bind(cssDistance));
}

/* Left */
.hermes-slide-left-enter-active,
.hermes-slide-left-leave-active {
  transition: all v-bind(cssDuration) cubic-bezier(0.16, 1, 0.3, 1);
}
.hermes-slide-left-enter-from {
  opacity: 0;
  transform: translateX(v-bind(cssDistance));
}
.hermes-slide-left-leave-to {
  opacity: 0;
  transform: translateX(calc(-1 * v-bind(cssDistance)));
}

/* Right */
.hermes-slide-right-enter-active,
.hermes-slide-right-leave-active {
  transition: all v-bind(cssDuration) cubic-bezier(0.16, 1, 0.3, 1);
}
.hermes-slide-right-enter-from {
  opacity: 0;
  transform: translateX(calc(-1 * v-bind(cssDistance)));
}
.hermes-slide-right-leave-to {
  opacity: 0;
  transform: translateX(v-bind(cssDistance));
}
</style>
