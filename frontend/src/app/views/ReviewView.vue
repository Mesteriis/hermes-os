<script setup lang="ts">
import { useI18n } from '../../platform/i18n'

const { t } = useI18n()
</script>

<template>
  <section class="panel settings-list-panel settings-primary-pane">
    <header class="panel-title-row">
      <div>
        <h2>{{ t('Review') }}</h2>
        <p>{{ t('Review UI removed after logic extraction. Rebuild pending new design language.') }}</p>
      </div>
    </header>
    <div class="empty-panel fill">
      <strong>{{ t('Review logic is preserved') }}</strong>
      <span>{{ t('Review item orchestration, promotion defaults and suggested entity review state remain in the extracted surface. This screen stays empty until the new review UI is rebuilt.') }}</span>
    </div>
  </section>
</template>
