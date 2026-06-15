<script setup lang="ts">
import { useI18n } from '../../../../platform/i18n'
import { shellBackgroundIds, type ShellBackgroundId } from '../../../../platform/theme/settings'

defineProps<{
	value: ShellBackgroundId
	title: string
	description: string
}>()

defineEmits<{
	change: [value: ShellBackgroundId]
}>()

const { t } = useI18n()

const labels: Record<ShellBackgroundId, string> = {
	none: 'No background',
	'network-mesh': 'Digital network',
	'data-stream': 'Data flow',
	'node-frame': 'Node grid',
	'eclipse-grid': 'Dark grid',
	'dna-blueprint': 'Connection blueprint',
	'forest-network': 'Green network',
	'forest-stream': 'Green flow',
	'knowledge-map': 'Knowledge map',
	'rune-gold': 'Warm accent',
	'rune-teal': 'Teal accent'
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
		<div class="background-option-grid">
			<button
				v-for="id in shellBackgroundIds"
				:key="id"
				type="button"
				class="background-option-btn"
				:class="{ active: value === id }"
				:aria-pressed="value === id"
				@click="$emit('change', id)"
			>
				<span class="shell-bg-preview" :class="`shell-bg-preview--${id}`" />
				<span>{{ t(labels[id]) }}</span>
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

.background-option-grid {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
	gap: 8px;
}

.background-option-btn {
	display: grid;
	gap: 6px;
	padding: 8px;
	border: 1px solid var(--hh-border);
	border-radius: var(--hh-radius-sm);
	background: var(--hh-surface-deep);
	cursor: pointer;
	text-align: center;
	font-size: 11px;
	color: var(--hh-text-secondary);
	transition: border-color 150ms ease, background 150ms ease;
}

.background-option-btn:hover,
.background-option-btn.active {
	border-color: var(--hh-accent);
}

.background-option-btn.active {
	background: var(--hh-hover-bg);
	color: var(--hh-accent);
}

.shell-bg-preview {
	display: block;
	width: 100%;
	height: 48px;
	border-radius: var(--hh-radius-xs);
	background: var(--hh-surface-deep);
	border: 1px solid var(--hh-border);
}

.shell-bg-preview--rune-teal { background: linear-gradient(135deg, #0f766e 0%, #042f2e 100%); }
.shell-bg-preview--rune-gold { background: linear-gradient(135deg, #b45309 0%, #451a03 100%); }
.shell-bg-preview--forest-network { background: linear-gradient(135deg, #065f46 0%, #022c22 100%); }
.shell-bg-preview--forest-stream { background: linear-gradient(135deg, #047857 0%, #064e3b 100%); }
.shell-bg-preview--knowledge-map { background: linear-gradient(135deg, #1e40af 0%, #1e1b4b 100%); }
.shell-bg-preview--eclipse-grid { background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%); }
.shell-bg-preview--network-mesh { background: linear-gradient(135deg, #334155 0%, #0f172a 100%); }
.shell-bg-preview--dna-blueprint { background: linear-gradient(135deg, #1e3a5f 0%, #0c1929 100%); }
.shell-bg-preview--data-stream { background: linear-gradient(135deg, #164e63 0%, #083344 100%); }
.shell-bg-preview--node-frame { background: linear-gradient(135deg, #0f172a 0%, #111827 100%); }
.shell-bg-preview--none { background: var(--hh-surface-deep); }
</style>
