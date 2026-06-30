<script setup lang="ts">
import { useI18n } from '../../platform/i18n'

const { t } = useI18n()
</script>

<template>
  <section class="panel settings-list-panel settings-primary-pane">
    <header class="panel-title-row">
      <div>
        <h2>{{ t('Event Tracing') }}</h2>
        <p>{{ t('Event tracing UI removed after logic extraction. Rebuild pending new design language.') }}</p>
      </div>
    </header>
    <div class="empty-panel fill">
      <strong>{{ t('Event tracing logic is preserved') }}</strong>
      <span>{{ t('Platform event trace queries, trace lookup keys and event envelope types remain in extracted TypeScript artifacts. This screen stays empty until the new event tracing UI is rebuilt.') }}</span>
    </div>
  </section>
</template>
