<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	as?: string
	side?: 'left' | 'right'
	width?: 'compact' | 'default' | 'wide'
	title?: string
	label?: string
	open?: boolean
	class?: string
}>(), {
	as: 'aside',
	side: 'left',
	width: 'default',
	open: true
})

const classes = computed(() => [
	'hermes-side-panel',
	`hermes-side-panel--${props.side}`,
	`hermes-side-panel--${props.width}`,
	props.class
])

const accessibleLabel = computed(() => props.label ?? props.title)
</script>

<template>
	<component v-if="open" :is="as" :class="classes" :aria-label="accessibleLabel">
		<header v-if="title || $slots.header" class="hermes-side-panel__header">
			<strong v-if="title" class="hermes-side-panel__title">{{ title }}</strong>
			<slot name="header" />
		</header>
		<div class="hermes-side-panel__body">
			<slot />
		</div>
		<footer v-if="$slots.footer" class="hermes-side-panel__footer">
			<slot name="footer" />
		</footer>
	</component>
</template>
