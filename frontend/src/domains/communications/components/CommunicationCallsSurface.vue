<script setup lang="ts">
import type { CommunicationCallsSurfaceModel } from './communicationDomainElements'
import './communicationDomainElements.css'
import CallWorkspace from './calls/CallWorkspace.vue'

defineProps<{
  surface: CommunicationCallsSurfaceModel
}>()
</script>

<template>
	<CallWorkspace :surface="surface" />
</template>
