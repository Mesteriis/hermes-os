<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	as?: string
	size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
	tone?: 'default' | 'strong' | 'muted' | 'soft' | 'accent' | 'danger'
	weight?: 'regular' | 'medium' | 'semibold' | 'bold'
	truncate?: boolean
	class?: string
}>(), {
	as: 'span',
	size: 'md',
	tone: 'default',
	weight: 'regular',
	truncate: false
})

const classes = computed(() => [
	'hermes-text',
	`hermes-text--${props.size}`,
	`hermes-text--${props.tone}`,
	`hermes-text--${props.weight}`,
	{ 'hermes-text--truncate': props.truncate },
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
