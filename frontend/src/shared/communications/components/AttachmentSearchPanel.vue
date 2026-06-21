<script setup lang="ts">
import AttachmentSearchPanel from '../../../domains/communications/components/AttachmentSearchPanel.vue'

const props = defineProps<{
  accountId: string | null
}>()
</script>

<template>
  <AttachmentSearchPanel :account-id="props.accountId" />
</template>
