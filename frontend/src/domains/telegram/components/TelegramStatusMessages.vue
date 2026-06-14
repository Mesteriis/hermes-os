<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'

const { t } = useI18n()

defineProps<{
  actionMessage: string
  error: string
}>()
</script>

<template>
  <p v-if="actionMessage" class="setup-state success">{{ actionMessage }}</p>
  <p v-if="error" class="inline-error">{{ error }}</p>
</template>

<style scoped>
.setup-state {
  padding: 8px 16px;
  margin: 0;
  font-size: 13px;
  border-radius: 6px;
}
.success {
  background: var(--color-success-bg, #e6f7e6);
  color: var(--color-success-text, #2e7d32);
}
.inline-error {
  padding: 8px 16px;
  margin: 0;
  font-size: 13px;
  background: var(--color-error-bg, #fdecea);
  color: var(--color-error-text, #c62828);
  border-radius: 6px;
}
</style>
