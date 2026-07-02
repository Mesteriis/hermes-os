<script setup lang="ts">
import { computed } from 'vue'

type SurfaceTone = 'default' | 'muted' | 'raised' | 'deep'
type SurfacePadding = 'none' | 'sm' | 'md' | 'lg'
type SurfaceRadius = 'none' | 'sm' | 'md' | 'lg'

const props = withDefaults(defineProps<{
	as?: string
	tone?: SurfaceTone
	padding?: SurfacePadding
	radius?: SurfaceRadius
	bordered?: boolean
	clip?: boolean
	class?: string
}>(), {
	as: 'section',
	tone: 'default',
	padding: 'md',
	radius: 'md',
	bordered: true,
	clip: false
})

const classes = computed(() => [
	'hermes-surface',
	`hermes-surface--${props.tone}`,
	`hermes-surface--padding-${props.padding}`,
	`hermes-surface--radius-${props.radius}`,
	props.bordered && 'hermes-surface--bordered',
	props.clip && 'hermes-surface--clip',
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
