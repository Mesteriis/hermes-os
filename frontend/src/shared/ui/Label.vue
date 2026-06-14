<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  htmlFor?: string
  class?: string
}>(), {})

const classes = computed(() => ['hermes-label', props.class])
</script>

<template>
  <label :class="classes" :for="htmlFor">
    <slot />
  </label>
</template>

<style scoped>
.hermes-label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-primary);
  line-height: 1.2;
}
</style>
