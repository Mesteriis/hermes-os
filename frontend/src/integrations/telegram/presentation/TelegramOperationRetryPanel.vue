<script setup lang="ts">
import type { TelegramOperationRetryModel } from './telegramOperationRetryModel'
import './telegramOperationRetryPanel.css'

defineProps<{ model: TelegramOperationRetryModel }>()

const emit = defineEmits<{
	retry: []
	updateOperationId: [value: string]
}>()
</script>

<template>
	<section class="telegram-operation-retry">
		<header>
			<div>
				<span>Operation recovery</span>
				<h2>Retry a failed provider command</h2>
			</div>
		</header>
		<form @submit.prevent="emit('retry')">
			<label for="telegram-retry-operation-id">Operation ID</label>
			<div>
				<input
					id="telegram-retry-operation-id"
					:value="model.operationId"
					@input="emit('updateOperationId', ($event.target as HTMLInputElement).value)"
				>
				<button
					type="submit"
					:disabled="!model.operationId.trim() || !model.canRetry || model.pending"
				>
					{{ model.pending ? 'Retrying…' : 'Retry now' }}
				</button>
			</div>
		</form>
		<small role="status">{{ model.statusMessage }}</small>
	</section>
</template>
