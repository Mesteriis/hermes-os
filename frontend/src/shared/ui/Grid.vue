<script setup lang="ts">
import { computed } from 'vue'
import type { LayoutGap } from './Layout.types'

const props = withDefaults(defineProps<{
	as?: string
	columns?: 'one' | 'two' | 'three' | 'four' | 'auto'
	gap?: LayoutGap
	align?: 'start' | 'center' | 'end' | 'stretch'
	dense?: boolean
	class?: string
}>(), {
	as: 'div',
	columns: 'auto',
	gap: 'md',
	align: 'stretch',
	dense: false
})

const classes = computed(() => [
	'hermes-grid',
	`hermes-grid--${props.columns}`,
	`hermes-grid--gap-${props.gap}`,
	`hermes-grid--align-${props.align}`,
	{ 'hermes-grid--dense': props.dense },
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
