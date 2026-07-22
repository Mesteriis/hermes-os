<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { SettingsOverviewCard } from '../queries/settingsPagePresentation'

defineProps<{
  cards: SettingsOverviewCard[]
  canReconnect: boolean
}>()

const emit = defineEmits<{
  reconnect: []
}>()

const { t } = useI18n()

function handleReconnect(): void {
  emit('reconnect')
}
</script>

<template>
  <section class="settings-status-strip" :aria-label="t('Settings overview')">
    <article
      v-for="card in cards"
      :key="card.id"
      class="settings-status-tile"
      :class="`tone-${card.tone}`"
    >
      <Icon :icon="card.icon" />
      <span>{{ t(card.label) }}</span>
      <strong>{{ card.value }}</strong>
      <button
        v-if="card.id === 'realtime' && canReconnect"
        type="button"
        class="icon-button"
        :aria-label="t('Reconnect')"
        @click="handleReconnect"
      >
        <Icon icon="tabler:refresh" />
      </button>
    </article>
  </section>
</template>
