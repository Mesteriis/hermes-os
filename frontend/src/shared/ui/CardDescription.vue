<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['hermes-card-description', props.class])
</script>

<template>
  <p :class="classes">
    <slot />
  </p>
</template>

<style scoped>
.hermes-card-description {
  font-size: 0.8125rem;
  color: var(--hh-text-muted);
  margin: 0;
  line-height: 1.4;
}
</style>
