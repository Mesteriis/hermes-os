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
	inline?: boolean
	class?: string
}>(), {
	as: 'div',
	direction: 'row',
	gap: 'md',
	align: 'stretch',
	justify: 'start',
	wrap: false,
	inline: false
})

const classes = computed(() => [
	'hermes-flex',
	`hermes-flex--${props.direction}`,
	`hermes-flex--gap-${props.gap}`,
	`hermes-flex--align-${props.align}`,
	`hermes-flex--justify-${props.justify}`,
	{
		'hermes-flex--wrap': props.wrap,
		'hermes-flex--inline': props.inline
	},
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
