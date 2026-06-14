<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'

const { t } = useI18n()

const smartCollections = [
  'Recently Added 48', 'Recently Opened 24', 'Important 32',
  'Shared with Me 18', 'Requires Review 7', 'Contracts & Legal 23', 'Financial 15'
]
const myFolders = [
  'Hermes Hub', 'Projects', 'Personal', 'Work', 'Archive 2024', 'Clients', 'References'
]
</script>

<template>
  <div class="widget-frame documents-navigation-panel">
    <aside class="left-panels">
      <section class="panel info-card">
        <h2>{{ t('Smart Collections') }}</h2>
        <div v-for="item in smartCollections" :key="item" class="collection-row">
          {{ t(item) }}
        </div>
      </section>
      <section class="panel info-card">
        <h2>{{ t('My Folders') }}</h2>
        <div v-for="folder in myFolders" :key="folder" class="collection-row">
          <Icon icon="tabler:folder" :size="16" />
          {{ t(folder) }}
        </div>
      </section>
    </aside>
  </div>
</template>
