<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	as?: string
	label?: string
	layer?: 'popover' | 'dialog' | 'toast'
	passive?: boolean
	class?: string
}>(), {
	as: 'div',
	layer: 'dialog',
	passive: false
})

const classes = computed(() => [
	'hermes-overlay-host',
	`hermes-overlay-host--${props.layer}`,
	{ 'hermes-overlay-host--passive': props.passive },
	props.class
])
</script>

<template>
	<component :is="as" :class="classes" :aria-label="label">
		<slot />
	</component>
</template>
