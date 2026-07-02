<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	label?: string
	active?: boolean
	class?: string
}>(), {
	label: 'Typing',
	active: true
})

const classes = computed(() => [
	'hermes-typing-indicator',
	{ 'hermes-typing-indicator--active': props.active },
	props.class
])
</script>

<template>
	<span :class="classes" role="status" :aria-label="label">
		<span class="hermes-typing-indicator__dot" aria-hidden="true" />
		<span class="hermes-typing-indicator__dot" aria-hidden="true" />
		<span class="hermes-typing-indicator__dot" aria-hidden="true" />
		<span class="hermes-sr-only">{{ label }}</span>
	</span>
</template>
