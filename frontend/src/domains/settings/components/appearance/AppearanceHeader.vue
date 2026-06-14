<script setup lang="ts">
defineProps<{
	title: string
	description: string
	isSaving: boolean
	saveStateLabel: string
	persistenceError: string
}>()

defineEmits<{
	reset: []
}>()
</script>

<template>
	<header class="panel-title-row">
		<div>
			<h2>{{ title }}</h2>
			<p>{{ description }}</p>
		</div>
			<div class="appearance-settings-actions">
				<button type="button" class="hermes-btn hermes-btn--outline" @click="$emit('reset')">
					Default
				</button>
				<div class="appearance-save-status">
					<span
						class="appearance-save-state"
						:class="{ 'appearance-save-state--warning': persistenceError }"
					>
						{{ isSaving ? 'Saving' : saveStateLabel }}
					</span>
					<p v-if="persistenceError" class="appearance-save-message">
						{{ persistenceError }}
					</p>
				</div>
			</div>
		</header>
	</template>

<style scoped>
.appearance-settings-actions {
	display: flex;
	flex-wrap: wrap;
	justify-content: flex-end;
	gap: 8px;
}

.appearance-save-status {
	display: flex;
	flex-direction: column;
	align-items: flex-end;
	gap: 4px;
	max-width: 320px;
}

.appearance-save-state {
	display: inline-flex;
	align-items: center;
	min-height: 34px;
	border: 1px solid rgba(111, 205, 195, 0.12);
	border-radius: var(--hh-radius-control, 6px);
	background: rgba(2, 12, 16, 0.48);
	color: var(--hh-text-muted);
	font-size: 12px;
	font-weight: 720;
	padding: 0 12px;
}

.appearance-save-state--warning {
	border-color: rgba(242, 184, 75, 0.34);
	background: rgba(92, 56, 8, 0.34);
	color: #f8d99c;
}

.appearance-save-message {
	margin: 0;
	color: #f8d99c;
	font-size: 12px;
	line-height: 1.35;
	text-align: right;
}
</style>
