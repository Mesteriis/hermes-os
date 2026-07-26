<script setup lang="ts">
import TelegramOperationalPage from '../presentation/TelegramOperationalPage.vue'
import { useTelegramOperationalPage } from '../queries/useTelegramOperationalPage'

const props = defineProps<{ canSend: boolean }>()
const surface = useTelegramOperationalPage(() => props.canSend)
</script>

<template>
	<TelegramOperationalPage
		:model="surface.model.value"
		@load="surface.loadChats"
		@select-chat="surface.selectChat"
		@send="surface.send"
		@update-account-id="surface.updateAccountId"
		@update-draft="surface.updateDraft"
	/>
</template>
