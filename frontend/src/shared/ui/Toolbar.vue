<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	as?: string
	orientation?: 'horizontal' | 'vertical'
	density?: 'compact' | 'default' | 'comfortable'
	label?: string
	class?: string
}>(), {
	as: 'div',
	orientation: 'horizontal',
	density: 'default'
})

const classes = computed(() => [
	'hermes-toolbar',
	`hermes-toolbar--${props.orientation}`,
	`hermes-toolbar--${props.density}`,
	props.class
])
</script>

<template>
	<component
		:is="as"
		:class="classes"
		role="toolbar"
		:aria-label="label"
		:aria-orientation="orientation"
	>
		<slot />
	</component>
</template>
