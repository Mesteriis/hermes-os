<script setup lang="ts">
import { computed } from 'vue'
import './SettingsSurfacePlaceholderPanel.css'

const props = defineProps<{
	title: string
	description: string
	capabilityIds: readonly string[]
}>()

const capabilityRows = computed(() => {
	const rows: string[] = []
	for (const capabilityId of props.capabilityIds) {
		rows.push(capabilityId)
	}
	return rows
})
</script>

<template>
	<section class="settings-surface-placeholder">
		<header class="settings-surface-placeholder__header">
			<strong>{{ title }}</strong>
			<p>{{ description }}</p>
		</header>
		<section class="settings-surface-placeholder__content">
			<h3>Ready composition entrypoints</h3>
			<ul class="settings-surface-placeholder__list">
				<li v-for="capability in capabilityRows" :key="capability">{{ capability }}</li>
			</ul>
			<p class="settings-surface-placeholder__note">
				The exact owner runtime and adapter must be admitted before executable controls can appear here.
			</p>
		</section>
	</section>
</template>
