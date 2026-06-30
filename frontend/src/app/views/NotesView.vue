<script setup lang="ts">
import { useI18n } from '../../platform/i18n'

const { t } = useI18n()
</script>

<template>
  <section class="panel settings-list-panel settings-primary-pane">
    <header class="panel-title-row">
      <div>
        <h2>{{ t('Notes') }}</h2>
        <p>{{ t('Notes UI removed after logic extraction. Rebuild pending new design language.') }}</p>
      </div>
    </header>
    <div class="empty-panel fill">
      <strong>{{ t('Notes logic is preserved') }}</strong>
      <span>{{ t('Note queries, fallback seed notes and notes filter state remain in the extracted surface and stores. This screen stays empty until the new notes UI is rebuilt.') }}</span>
    </div>
  </section>
</template>
