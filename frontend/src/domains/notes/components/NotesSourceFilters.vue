<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'

const { t } = useI18n()

defineProps<{
  activeSources: string[]
  activeTags: string[]
}>()

const emit = defineEmits<{
  'toggle-source': [source: string]
  'toggle-tag': [tag: string]
}>()

const sources = ['Apple Notes', 'Obsidian', 'Gmail', 'Anytype', 'Outlook']
const tags = ['#project', '#research', '#meeting', '#idea', '#reference', '#partnership']
</script>

<template>
  <aside class="left-panels">
    <div class="widget-frame">
      <section class="panel info-card">
        <h2>{{ t('Sources') }}</h2>
        <label v-for="src in sources" :key="src" class="mini-check">
          <input
            type="checkbox"
            :checked="activeSources.includes(src)"
            @change="emit('toggle-source', src)"
          />
          {{ t(src) }}
        </label>
      </section>
    </div>
    <div class="widget-frame">
      <section class="panel info-card">
        <h2>{{ t('Tags') }}</h2>
        <label v-for="tag in tags" :key="tag" class="mini-check">
          <input
            type="checkbox"
            :checked="activeTags.includes(tag)"
            @change="emit('toggle-tag', tag)"
          />
          {{ t(tag) }}
        </label>
      </section>
    </div>
  </aside>
</template>
