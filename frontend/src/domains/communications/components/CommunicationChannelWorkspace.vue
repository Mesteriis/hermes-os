<script setup lang="ts">
import ChannelWorkspace from './channels/ChannelWorkspace.vue'
import type { CommunicationChannelWorkspaceModel } from './communicationDomainElements'

defineProps<{
  workspace: CommunicationChannelWorkspaceModel
}>()
</script>

<template>
	<ChannelWorkspace :workspace="workspace" />
</template>
