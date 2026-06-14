<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'

const { t } = useI18n()

const sources = [
  { name: 'Google Drive', count: '1,243', icon: 'tabler:brand-google-drive' },
  { name: 'OneDrive', count: '812', icon: 'tabler:brand-office' },
  { name: 'Dropbox', count: '342', icon: 'tabler:brand-dropbox' },
  { name: 'Notion', count: '256', icon: 'tabler:brand-notion' }
]
</script>

<template>
  <div class="widget-frame documents-source-cards-panel">
    <div class="document-source-cards">
      <div v-for="src in sources" :key="src.name" class="source-card">
        <span class="round-icon">
          <Icon :icon="src.icon" :size="24" />
        </span>
        <strong>{{ t(src.name) }}</strong>
        <em>{{ src.count }}</em>
      </div>
    </div>
  </div>
</template>
