<script setup lang="ts">
import ZulipSettingsPanel from '../../integrations/zulip/components/ZulipSettingsPanel.vue'
import type { ProviderAccount } from './settingsBridge'

defineProps<{
  selectedAccount?: ProviderAccount | null
}>()
</script>

<template>
  <ZulipSettingsPanel :selected-account="selectedAccount" />
</template>
