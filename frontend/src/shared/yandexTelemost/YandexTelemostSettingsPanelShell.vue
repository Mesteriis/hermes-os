<script setup lang="ts">
import YandexTelemostSettingsPanel from '../../integrations/yandexTelemost/components/YandexTelemostSettingsPanel.vue'
import type { ProviderAccount } from './settingsBridge'

defineProps<{
  selectedAccount?: ProviderAccount | null
}>()
</script>

<template>
  <YandexTelemostSettingsPanel :selected-account="selectedAccount" />
</template>
