<script setup lang="ts">
defineProps<{
  actionMessage: string
  error: string
}>()
</script>

<template>
  <p v-if="actionMessage" class="setup-state success">{{ actionMessage }}</p>
  <p v-if="error" class="inline-error">{{ error }}</p>
</template>
