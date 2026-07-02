<script setup lang="ts">
import { computed } from 'vue'
import type { LayoutAlign, LayoutDirection, LayoutGap, LayoutJustify } from './Layout.types'

const props = withDefaults(defineProps<{
	as?: string
	direction?: LayoutDirection
	gap?: LayoutGap
	align?: LayoutAlign
	justify?: LayoutJustify
	wrap?: boolean
	class?: string
}>(), {
	as: 'div',
	direction: 'column',
	gap: 'md',
	align: 'stretch',
	justify: 'start',
	wrap: false
})

const classes = computed(() => [
	'hermes-stack',
	`hermes-stack--${props.direction}`,
	`hermes-stack--gap-${props.gap}`,
	`hermes-stack--align-${props.align}`,
	`hermes-stack--justify-${props.justify}`,
	{ 'hermes-stack--wrap': props.wrap },
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
