<script setup lang="ts">
defineProps<{
	id: string
	label: string
	description: string
	value: number
	min: number
	max: number
	step: number
	unit: string
}>()

defineEmits<{
	preview: [value: number]
	commit: []
}>()
</script>

<template>
	<section class="appearance-section">
		<header>
			<div>
				<h3>{{ label }}</h3>
				<p>{{ description }}</p>
			</div>
			<strong>{{ value }}{{ unit }}</strong>
		</header>
		<input
			:id="id"
			type="range"
			:min="min"
			:max="max"
			:step="step"
			:value="value"
			:aria-label="label"
			@input="$emit('preview', Number(($event.target as HTMLInputElement).value))"
			@change="$emit('commit')"
		/>
	</section>
</template>

<style scoped>
.appearance-section {
	display: grid;
	gap: 12px;
	padding: var(--hh-space-panel);
	border-top: 1px solid var(--hh-border);
}

.appearance-section header {
	display: flex;
	align-items: baseline;
	justify-content: space-between;
	gap: 12px;
}

.appearance-section header h3 {
	margin: 0;
	font-size: 13px;
	font-weight: 680;
	color: var(--hh-text-primary);
}

.appearance-section header p {
	margin: 2px 0 0;
	font-size: 11px;
	color: var(--hh-text-muted);
}

.appearance-section header strong {
	font-size: 13px;
	font-weight: 720;
	color: var(--hh-accent);
	white-space: nowrap;
}

input[type="range"] {
	width: 100%;
	height: 6px;
	-webkit-appearance: none;
	appearance: none;
	background: var(--hh-hover-bg);
	border-radius: 3px;
	outline: none;
	cursor: pointer;
}

input[type="range"]::-webkit-slider-thumb {
	-webkit-appearance: none;
	width: 18px;
	height: 18px;
	border-radius: 50%;
	background: var(--hh-accent);
	border: 2px solid var(--hh-surface-panel);
	cursor: pointer;
	box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
}
</style>
