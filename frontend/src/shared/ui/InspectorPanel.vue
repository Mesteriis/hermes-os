<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	as?: string
	title?: string
	description?: string
	label?: string
	density?: 'compact' | 'default'
	class?: string
}>(), {
	as: 'aside',
	density: 'default'
})

const classes = computed(() => [
	'hermes-inspector-panel',
	`hermes-inspector-panel--${props.density}`,
	props.class
])

const accessibleLabel = computed(() => props.label ?? props.title)
</script>

<template>
	<component :is="as" :class="classes" :aria-label="accessibleLabel">
		<header v-if="title || description || $slots.header" class="hermes-inspector-panel__header">
			<strong v-if="title" class="hermes-inspector-panel__title">{{ title }}</strong>
			<span v-if="description" class="hermes-inspector-panel__description">{{ description }}</span>
			<slot name="header" />
		</header>
		<div class="hermes-inspector-panel__body">
			<slot />
		</div>
		<footer v-if="$slots.footer" class="hermes-inspector-panel__footer">
			<slot name="footer" />
		</footer>
	</component>
</template>
