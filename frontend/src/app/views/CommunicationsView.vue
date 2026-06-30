<script setup lang="ts">
import { useI18n } from '../../platform/i18n'

const { t } = useI18n()
</script>

<template>
  <section class="panel settings-list-panel settings-primary-pane">
    <header class="panel-title-row">
      <div>
        <h2>{{ t('Communications') }}</h2>
        <p>{{ t('Communications UI removed after logic extraction. Rebuild pending new design language.') }}</p>
      </div>
    </header>
    <div class="empty-panel fill">
      <strong>{{ t('Communications logic is preserved') }}</strong>
      <span>{{ t('Message queries, thread orchestration, outbox state and compose flows remain in the extracted surface. This screen stays empty until the new communications UI is rebuilt.') }}</span>
    </div>
  </section>
</template>
