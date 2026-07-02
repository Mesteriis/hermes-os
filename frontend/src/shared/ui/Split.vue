<script setup lang="ts">
import { computed } from 'vue'
import type { LayoutGap, LayoutOrientation } from './Layout.types'

const props = withDefaults(defineProps<{
	as?: string
	orientation?: LayoutOrientation
	ratio?: 'balanced' | 'start' | 'end'
	gap?: LayoutGap
	class?: string
}>(), {
	as: 'div',
	orientation: 'horizontal',
	ratio: 'balanced',
	gap: 'md'
})

const classes = computed(() => [
	'hermes-split',
	`hermes-split--${props.orientation}`,
	`hermes-split--ratio-${props.ratio}`,
	`hermes-split--gap-${props.gap}`,
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<div class="hermes-split-pane hermes-split-pane--primary">
			<slot name="primary">
				<slot />
			</slot>
		</div>
		<div v-if="$slots.secondary" class="hermes-split-pane hermes-split-pane--secondary">
			<slot name="secondary" />
		</div>
	</component>
</template>
