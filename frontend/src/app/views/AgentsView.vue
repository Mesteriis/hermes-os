<script setup lang="ts">
import { useI18n } from '../../platform/i18n'

const { t } = useI18n()
</script>

<template>
  <section class="panel settings-list-panel settings-primary-pane">
    <header class="panel-title-row">
      <div>
        <h2>{{ t('Agents') }}</h2>
        <p>{{ t('Agents UI removed after logic extraction. Rebuild pending new design language.') }}</p>
      </div>
    </header>
    <div class="empty-panel fill">
      <strong>{{ t('Agents logic is preserved') }}</strong>
      <span>{{ t('AI workspace orchestration, agent cards and action workflows remain in the extracted surface and stores. This screen stays empty until the new agents UI is rebuilt.') }}</span>
    </div>
  </section>
</template>
