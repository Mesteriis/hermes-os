<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'

const { t } = useI18n()
</script>

<template>
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Note Insights') }}</h2>
      <p>{{ t('AI generated summaries and connections across your notes will appear here.') }}</p>
    </section>
  </div>
</template>
