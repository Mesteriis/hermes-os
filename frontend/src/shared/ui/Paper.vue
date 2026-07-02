<script setup lang="ts">
import { computed } from 'vue'

type PaperTone = 'default' | 'muted' | 'raised' | 'deep'
type PaperPadding = 'none' | 'sm' | 'md' | 'lg'
type PaperRadius = 'none' | 'sm' | 'md' | 'lg'

const props = withDefaults(defineProps<{
	as?: string
	tone?: PaperTone
	padding?: PaperPadding
	radius?: PaperRadius
	bordered?: boolean
	clip?: boolean
	class?: string
}>(), {
	as: 'article',
	tone: 'default',
	padding: 'md',
	radius: 'md',
	bordered: true,
	clip: false
})

const classes = computed(() => [
	'hermes-paper',
	`hermes-paper--${props.tone}`,
	`hermes-paper--padding-${props.padding}`,
	`hermes-paper--radius-${props.radius}`,
	props.bordered && 'hermes-paper--bordered',
	props.clip && 'hermes-paper--clip',
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
