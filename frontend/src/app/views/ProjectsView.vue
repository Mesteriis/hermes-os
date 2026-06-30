<script setup lang="ts">
import { useI18n } from '../../platform/i18n'

const { t } = useI18n()
</script>

<template>
  <section class="panel settings-list-panel settings-primary-pane">
    <header class="panel-title-row">
      <div>
        <h2>{{ t('Projects') }}</h2>
        <p>{{ t('Projects UI removed after logic extraction. Rebuild pending new design language.') }}</p>
      </div>
    </header>
    <div class="empty-panel fill">
      <strong>{{ t('Projects logic is preserved') }}</strong>
      <span>{{ t('Project queries, selection orchestration and project summary formatting remain in the extracted surface and stores. This screen stays empty until the new projects UI is rebuilt.') }}</span>
    </div>
  </section>
</template>
