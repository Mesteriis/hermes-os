<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'

const { t } = useI18n()
</script>

<template>
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Document Insights') }}</h2>
      <p>{{ t('AI analysis results will appear here when document processing is complete.') }}</p>
    </section>
  </div>
</template>
