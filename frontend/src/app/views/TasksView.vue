<script setup lang="ts">
import { useI18n } from '../../platform/i18n'

const { t } = useI18n()
</script>

<template>
  <section class="panel settings-list-panel settings-primary-pane">
    <header class="panel-title-row">
      <div>
        <h2>{{ t('Tasks') }}</h2>
        <p>{{ t('Tasks UI removed after logic extraction. Rebuild pending new design language.') }}</p>
      </div>
    </header>
    <div class="empty-panel fill">
      <strong>{{ t('Tasks logic is preserved') }}</strong>
      <span>{{ t('Task queries, review orchestration and context review state remain in the extracted surface. This screen stays empty until the new tasks UI is rebuilt.') }}</span>
    </div>
  </section>
</template>
