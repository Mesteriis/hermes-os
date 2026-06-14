<script setup lang="ts">
defineProps<{
  isLoading: boolean
  hasError: boolean
  hasRail: boolean
}>()
</script>

<template>
  <section class="communications-workbench" :class="{ 'has-rail': hasRail, 'is-loading': isLoading, 'has-error': hasError }">
    <slot name="list" />
    <slot name="detail" />
    <slot v-if="hasRail" name="rail" />
  </section>
</template>

<style scoped>
.communications-workbench {
  flex: 1;
  display: grid;
  grid-template-columns: 280px 1fr;
  overflow: hidden;
}

.communications-workbench.has-rail {
  grid-template-columns: 240px 1fr 280px;
}
</style>
