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
