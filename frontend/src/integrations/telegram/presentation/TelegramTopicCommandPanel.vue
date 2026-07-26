<script setup lang="ts">
import type { TelegramTopicCommandModel } from '../queries/useTelegramTopicCommands'

defineProps<{ model: TelegramTopicCommandModel }>()

const emit = defineEmits<{
	closeTopic: [active: boolean]
	createTopic: []
	refreshParticipants: []
	refreshTopics: []
	searchMessages: []
	updateProviderSearchQuery: [value: string]
	updateTopicId: [value: string]
	updateTopicTitle: [value: string]
}>()
</script>

<template>
	<section class="telegram-command-panel">
		<header><h3>Provider fetch & topics</h3></header>
		<label for="telegram-provider-search">Provider message search</label>
		<div>
			<input
				id="telegram-provider-search"
				type="search"
				:value="model.providerSearchQuery"
				@input="emit('updateProviderSearchQuery', ($event.target as HTMLInputElement).value)"
			>
			<button type="button" :disabled="!model.hasChat || !model.providerSearchQuery.trim() || !model.canCommand || model.pending" @click="emit('searchMessages')">Request</button>
		</div>
		<div class="telegram-command-panel__actions">
			<button type="button" :disabled="!model.hasChat || !model.canCommand || model.pending" @click="emit('refreshParticipants')">Fetch participants</button>
			<button type="button" :disabled="!model.hasChat || !model.canCommand || model.pending" @click="emit('refreshTopics')">Fetch topics</button>
		</div>
		<label for="telegram-topic-title">New topic title</label>
		<div>
			<input
				id="telegram-topic-title"
				:value="model.topicTitle"
				@input="emit('updateTopicTitle', ($event.target as HTMLInputElement).value)"
			>
			<button type="button" :disabled="!model.hasChat || !model.topicTitle.trim() || !model.canCommand || model.pending" @click="emit('createTopic')">Create</button>
		</div>
		<label for="telegram-topic-id">Topic ID</label>
		<div>
			<input
				id="telegram-topic-id"
				:value="model.topicId"
				@input="emit('updateTopicId', ($event.target as HTMLInputElement).value)"
			>
			<button type="button" :disabled="!model.hasChat || !model.topicId.trim() || !model.canCommand || model.pending" @click="emit('closeTopic', true)">Close</button>
			<button type="button" :disabled="!model.hasChat || !model.topicId.trim() || !model.canCommand || model.pending" @click="emit('closeTopic', false)">Reopen</button>
		</div>
		<small role="status">{{ model.statusMessage }}</small>
	</section>
</template>
