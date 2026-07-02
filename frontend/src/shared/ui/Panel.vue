<script setup lang="ts">
import { computed } from 'vue'

type PanelTone = 'default' | 'muted' | 'raised' | 'deep'
type PanelPadding = 'none' | 'sm' | 'md' | 'lg'
type PanelRadius = 'none' | 'sm' | 'md' | 'lg'

const props = withDefaults(defineProps<{
	as?: string
	tone?: PanelTone
	padding?: PanelPadding
	radius?: PanelRadius
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
	'hermes-panel',
	`hermes-panel--${props.tone}`,
	`hermes-panel--padding-${props.padding}`,
	`hermes-panel--radius-${props.radius}`,
	props.bordered && 'hermes-panel--bordered',
	props.clip && 'hermes-panel--clip',
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
