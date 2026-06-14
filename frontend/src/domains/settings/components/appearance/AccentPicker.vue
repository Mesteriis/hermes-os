<script setup lang="ts">
import { accentColorIds, type AccentColorId } from '../../../../platform/theme/settings'

defineProps<{
	value: AccentColorId
	title: string
	description: string
}>()

defineEmits<{
	change: [value: AccentColorId]
}>()

const labels: Record<AccentColorId, string> = {
	teal: 'Teal',
	cyan: 'Cyan',
	blue: 'Blue',
	violet: 'Violet',
	amber: 'Amber',
	rose: 'Rose'
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
		<div class="accent-option-grid">
			<button
				v-for="id in accentColorIds"
				:key="id"
				type="button"
				class="accent-option-btn"
				:class="{ active: value === id }"
				:aria-pressed="value === id"
				@click="$emit('change', id)"
			>
				<span class="accent-swatch" :class="`accent-swatch--${id}`" />
				<span>{{ labels[id] }}</span>
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

.accent-option-grid {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
	gap: 8px;
}

.accent-option-btn {
	display: flex;
	align-items: center;
	gap: 6px;
	padding: 6px 8px;
	border: 1px solid var(--hh-border);
	border-radius: var(--hh-radius-sm);
	background: transparent;
	cursor: pointer;
	font-size: 11px;
	color: var(--hh-text-secondary);
	transition: border-color 150ms ease, background 150ms ease;
}

.accent-option-btn:hover,
.accent-option-btn.active {
	border-color: var(--hh-accent);
}

.accent-option-btn.active {
	background: var(--hh-hover-bg);
	color: var(--hh-accent);
}

.accent-swatch {
	display: inline-block;
	width: 14px;
	height: 14px;
	border-radius: 50%;
	border: 1px solid var(--hh-border);
	flex-shrink: 0;
}

.accent-swatch--teal { background: #14b8a6; }
.accent-swatch--cyan { background: #06b6d4; }
.accent-swatch--blue { background: #3b82f6; }
.accent-swatch--violet { background: #8b5cf6; }
.accent-swatch--amber { background: #f59e0b; }
.accent-swatch--rose { background: #f43f5e; }
</style>
