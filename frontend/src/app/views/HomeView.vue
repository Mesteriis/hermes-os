<script setup lang="ts">
import { useI18n } from '../../platform/i18n'

const { t } = useI18n()
</script>

<template>
  <section class="panel settings-list-panel settings-primary-pane">
    <header class="panel-title-row">
      <div>
        <h2>{{ t('Home') }}</h2>
        <p>{{ t('Home UI removed after logic extraction. Rebuild pending new design language.') }}</p>
      </div>
    </header>
    <div class="empty-panel fill">
      <strong>{{ t('Home logic is preserved') }}</strong>
      <span>{{ t('Dashboard stats, message feed derivations and people-talked summaries remain in the extracted surface. This screen stays empty until the new home UI is rebuilt.') }}</span>
    </div>
  </section>
</template>
