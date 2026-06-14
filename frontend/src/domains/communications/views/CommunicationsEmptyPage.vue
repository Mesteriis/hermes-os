<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'

const { t } = useI18n()
</script>

<template>
  <div class="empty-section">
    <Icon icon="tabler:messages-off" class="empty-icon" />
    <h3 class="empty-label">{{ t('communications.empty.title') }}</h3>
    <p class="empty-description">
      {{ t('communications.empty.description') }}
    </p>
  </div>
</template>

<style scoped>
.empty-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 2rem;
  color: var(--hh-text-secondary, #6b7280);
  text-align: center;
}

.empty-icon {
  width: 48px;
  height: 48px;
  margin-bottom: 1rem;
  opacity: 0.4;
}

.empty-label {
  font-size: 1.125rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: var(--hh-text-primary, #1f2937);
}

.empty-description {
  font-size: 0.875rem;
  max-width: 280px;
  line-height: 1.4;
}
</style>
