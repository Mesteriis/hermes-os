<script setup lang="ts">
import { computed } from 'vue'
import type { LayoutTone } from './Layout.types'

const props = withDefaults(defineProps<{
	as?: string
	placement?: 'top-start' | 'top-end' | 'bottom-start' | 'bottom-end'
	tone?: LayoutTone
	label?: string
	class?: string
}>(), {
	as: 'section',
	placement: 'bottom-end',
	tone: 'floating'
})

const classes = computed(() => [
	'hermes-floating-panel',
	`hermes-floating-panel--${props.placement}`,
	`hermes-floating-panel--${props.tone}`,
	props.class
])
</script>

<template>
	<component :is="as" :class="classes" :aria-label="label">
		<slot />
	</component>
</template>
