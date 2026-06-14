<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['hermes-card-title', props.class])
</script>

<template>
  <h3 :class="classes">
    <slot />
  </h3>
</template>

<style scoped>
.hermes-card-title {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--hh-text-primary);
  margin: 0;
  line-height: 1.3;
}
</style>
