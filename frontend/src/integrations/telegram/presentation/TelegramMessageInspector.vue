<script setup lang="ts">
import type {
	TelegramMessageInspectorModel,
	TelegramMessageInspectorRow,
} from './telegramMessageInspectorModel'
import './telegramMessageInspector.css'

defineProps<{ model: TelegramMessageInspectorModel }>()

const emit = defineEmits<{ inspect: [] }>()

function sectionTitle(label: string, rows: readonly TelegramMessageInspectorRow[]): string {
	return `${label} · ${rows.length}`
}
</script>

<template>
	<section class="telegram-message-inspector">
		<header>
			<div>
				<span>Message audit</span>
				<h2>Versions, lineage & provider state</h2>
				<small>{{ model.selectedMessageId || 'Select a message' }}</small>
			</div>
			<button
				type="button"
				:disabled="!model.selectedMessageId || !model.canQuery || model.pending"
				@click="emit('inspect')"
			>
				{{ model.pending ? 'Inspecting…' : 'Inspect message' }}
			</button>
		</header>
		<p v-if="model.statusMessage" role="status">{{ model.statusMessage }}</p>
		<div class="telegram-message-inspector__badges">
			<span v-for="item in model.overview" :key="item">{{ item }}</span>
		</div>
		<div class="telegram-message-inspector__sections">
			<details open>
				<summary>{{ sectionTitle('Versions', model.versions) }}</summary>
				<article v-for="item in model.versions" :key="item.id"><strong>{{ item.title }}</strong><small>{{ item.detail }}</small></article>
			</details>
			<details>
				<summary>{{ sectionTitle('Tombstones', model.tombstones) }}</summary>
				<article v-for="item in model.tombstones" :key="item.id"><strong>{{ item.title }}</strong><small>{{ item.detail }}</small></article>
			</details>
			<details>
				<summary>{{ sectionTitle('Mutations', model.mutations) }}</summary>
				<article v-for="item in model.mutations" :key="item.id"><strong>{{ item.title }}</strong><small>{{ item.detail }}</small></article>
			</details>
			<details>
				<summary>{{ sectionTitle('Reply chain', model.replyChain) }}</summary>
				<article v-for="item in model.replyChain" :key="item.id"><strong>{{ item.title }}</strong><small>{{ item.detail }}</small></article>
			</details>
			<details>
				<summary>{{ sectionTitle('Forward chain', model.forwardChain) }}</summary>
				<article v-for="item in model.forwardChain" :key="item.id"><strong>{{ item.title }}</strong><small>{{ item.detail }}</small></article>
			</details>
			<details>
				<summary>{{ sectionTitle('Reactions', model.reactions) }}</summary>
				<article v-for="item in model.reactions" :key="item.id"><strong>{{ item.title }}</strong><small>{{ item.detail }}</small></article>
			</details>
			<details>
				<summary>{{ sectionTitle('Command audit', model.commands) }}</summary>
				<article v-for="item in model.commands" :key="item.id"><strong>{{ item.title }}</strong><small>{{ item.detail }}</small></article>
			</details>
		</div>
	</section>
</template>
