<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['hermes-card-content', props.class])
</script>

<template>
  <div :class="classes">
    <slot />
  </div>
</template>

<style scoped>
.hermes-card-content {
  padding: 1.25rem;
}
</style>
