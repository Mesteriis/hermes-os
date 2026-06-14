<script setup lang="ts">
import { spacingDensityIds, type SpacingDensity } from '../../../../platform/theme/settings'

defineProps<{
	value: SpacingDensity
	title: string
	description: string
}>()

defineEmits<{
	change: [value: SpacingDensity]
}>()

const labels: Record<SpacingDensity, string> = {
	compact: 'Compact',
	normal: 'Normal',
	comfortable: 'Comfortable'
}
</script>

<template>
	<section class="appearance-section">
		<header>
			<div>
				<h3>{{ title }}</h3>
				<p>{{ description }}</p>
			</div>
		</header>
		<div class="density-options">
			<button
				v-for="id in spacingDensityIds"
				:key="id"
				type="button"
				class="density-option-btn"
				:class="{ active: value === id }"
				:aria-pressed="value === id"
				@click="$emit('change', id)"
			>
				{{ labels[id] }}
			</button>
		</div>
	</section>
</template>

<style scoped>
.appearance-section {
	display: grid;
	gap: 12px;
	padding: var(--hh-space-panel);
	border-top: 1px solid var(--hh-border);
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

.density-options {
	display: flex;
	flex-wrap: wrap;
	gap: 6px;
}

.density-option-btn {
	min-width: 96px;
	height: 34px;
	padding: 0 10px;
	border: 1px solid var(--hh-border);
	border-radius: var(--hh-radius-sm);
	background: transparent;
	color: var(--hh-text-secondary);
	font-size: 12px;
	font-weight: 620;
	cursor: pointer;
	transition: all 100ms ease;
}

.density-option-btn:hover,
.density-option-btn.active {
	border-color: var(--hh-accent);
}

.density-option-btn.active {
	background: color-mix(in srgb, var(--hh-accent) 15%, transparent);
	color: var(--hh-accent);
}
</style>
