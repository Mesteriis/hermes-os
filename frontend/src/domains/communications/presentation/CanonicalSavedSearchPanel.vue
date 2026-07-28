<script setup lang="ts">
import type { CanonicalSavedSearchPanelModel } from './canonicalSavedSearchPanelModel'
import './canonicalSavedSearchPanel.css'

defineProps<{ model: CanonicalSavedSearchPanelModel }>()

const emit = defineEmits<{
	create: []
	execute: [itemKey: string]
	loadMoreItems: []
	loadMoreResults: []
	remove: [itemKey: string]
	replace: [itemKey: string]
	selectMessage: [messageKey: string]
	updateDescription: [value: string]
	updateName: [value: string]
	updateScopeCurrentAccount: [value: boolean]
}>()
</script>

<template>
	<section class="canonical-saved-searches" aria-labelledby="canonical-saved-searches-title">
		<header>
			<div>
				<span>Private owner projection</span>
				<h2 id="canonical-saved-searches-title">Saved searches</h2>
				<p>Names and keyed exact-token definitions stay inside Communications.</p>
			</div>
			<span>{{ model.items.length }}</span>
		</header>

		<p
			v-if="model.status === 'unavailable'"
			class="canonical-saved-searches__state"
			role="status"
		>{{ model.statusMessage }}</p>

		<template v-else>
			<form class="canonical-saved-searches__create" @submit.prevent="emit('create')">
				<label>
					<span>Name</span>
					<input
						type="text"
						autocomplete="off"
						maxlength="128"
						required
						:value="model.name"
						@input="emit('updateName', ($event.target as HTMLInputElement).value)"
					>
				</label>
				<label>
					<span>Description</span>
					<input
						type="text"
						autocomplete="off"
						maxlength="512"
						:value="model.description"
						@input="emit('updateDescription', ($event.target as HTMLInputElement).value)"
					>
				</label>
				<label class="canonical-saved-searches__scope">
					<input
						type="checkbox"
						:checked="model.scopeCurrentAccount"
						:disabled="!model.canScopeToCurrentAccount"
						@change="emit(
							'updateScopeCurrentAccount',
							($event.target as HTMLInputElement).checked,
						)"
					>
					<span>Limit to selected canonical account</span>
				</label>
				<button type="submit" :disabled="model.busy || !model.name.trim()">
					Save current search
				</button>
			</form>

			<p
				v-if="model.statusMessage"
				class="canonical-saved-searches__state"
				:role="model.status === 'error' ? 'alert' : 'status'"
			>{{ model.statusMessage }}</p>

			<div class="canonical-saved-searches__grid">
				<article
					v-for="item in model.items"
					:key="item.key"
					:class="{ active: item.active }"
				>
					<header>
						<div>
							<strong>{{ item.name }}</strong>
							<small>{{ item.scopeLabel }}</small>
						</div>
						<span>{{ item.tokenLabel }}</span>
					</header>
					<p v-if="item.description">{{ item.description }}</p>
					<footer>
						<small>{{ item.revisionLabel }} · {{ item.updatedLabel }}</small>
						<div>
							<button type="button" :disabled="model.busy" @click="emit('execute', item.key)">
								Apply
							</button>
							<button type="button" :disabled="model.busy" @click="emit('replace', item.key)">
								Replace with current search
							</button>
							<button type="button" :disabled="model.busy" @click="emit('remove', item.key)">
								Delete
							</button>
						</div>
					</footer>
				</article>
			</div>

			<button
				v-if="model.hasMoreItems"
				type="button"
				class="canonical-saved-searches__more"
				:disabled="model.busy"
				@click="emit('loadMoreItems')"
			>Load more saved searches</button>

			<section
				v-if="model.results.length > 0"
				class="canonical-saved-searches__results"
				aria-labelledby="canonical-saved-search-results-title"
			>
				<header>
					<h3 id="canonical-saved-search-results-title">Saved-search results</h3>
					<span>{{ model.results.length }}</span>
				</header>
				<button
					v-for="result in model.results"
					:key="result.key"
					type="button"
					:class="{ selected: result.selected }"
					:aria-pressed="result.selected"
					@click="emit('selectMessage', result.messageKey)"
				>
					<strong>{{ result.evidenceLabel }}</strong>
					<span>{{ result.messageLabel }}</span>
					<span>{{ result.conversationLabel }}</span>
					<small>{{ result.observedAtLabel }} · {{ result.matchLabel }}</small>
				</button>
				<button
					v-if="model.hasMoreResults"
					type="button"
					class="canonical-saved-searches__more"
					:disabled="model.busy"
					@click="emit('loadMoreResults')"
				>Load more results</button>
			</section>
		</template>
	</section>
</template>
