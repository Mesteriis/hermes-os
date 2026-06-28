<script setup lang="ts">
import ZoomSettingsPanel from '../../integrations/zoom/components/ZoomSettingsPanel.vue'
import type { ProviderAccount } from './settingsBridge'

defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

defineEmits<{
  removed: []
}>()
</script>

<template>
  <ZoomSettingsPanel
    :selected-account="selectedAccount"
    @removed="$emit('removed')"
  />
</template>
