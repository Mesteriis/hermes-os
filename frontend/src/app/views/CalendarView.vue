<script setup lang="ts">
import { useI18n } from '../../platform/i18n'

const { t } = useI18n()
</script>

<template>
  <section class="panel settings-list-panel settings-primary-pane">
    <header class="panel-title-row">
      <div>
        <h2>{{ t('Calendar') }}</h2>
        <p>{{ t('Calendar UI removed after logic extraction. Rebuild pending new design language.') }}</p>
      </div>
    </header>
    <div class="empty-panel fill">
      <strong>{{ t('Calendar logic is preserved') }}</strong>
      <span>{{ t('Calendar queries, event creation orchestration and weekly brief state remain in the extracted surface. This screen stays empty until the new calendar UI is rebuilt.') }}</span>
    </div>
  </section>
</template>
