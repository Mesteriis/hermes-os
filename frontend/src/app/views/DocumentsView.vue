<script setup lang="ts">
import { useI18n } from '../../platform/i18n'

const { t } = useI18n()
</script>

<template>
  <section class="panel settings-list-panel settings-primary-pane">
    <header class="panel-title-row">
      <div>
        <h2>{{ t('Documents') }}</h2>
        <p>{{ t('Documents UI removed after logic extraction. Rebuild pending new design language.') }}</p>
      </div>
    </header>
    <div class="empty-panel fill">
      <strong>{{ t('Documents logic is preserved') }}</strong>
      <span>{{ t('Document queries and retry orchestration remain in the extracted surface. This screen stays empty until the new documents UI is rebuilt.') }}</span>
    </div>
  </section>
</template>
