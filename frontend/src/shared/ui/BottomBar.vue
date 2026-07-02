<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	as?: string
	label?: string
	density?: 'compact' | 'default'
	class?: string
}>(), {
	as: 'footer',
	density: 'default'
})

const classes = computed(() => [
	'hermes-bottom-bar',
	`hermes-bottom-bar--${props.density}`,
	props.class
])
</script>

<template>
	<component :is="as" :class="classes" :aria-label="label">
		<div v-if="$slots.start" class="hermes-bottom-bar__slot">
			<slot name="start" />
		</div>
		<div class="hermes-bottom-bar__main">
			<slot />
		</div>
		<div v-if="$slots.end" class="hermes-bottom-bar__slot">
			<slot name="end" />
		</div>
	</component>
</template>
