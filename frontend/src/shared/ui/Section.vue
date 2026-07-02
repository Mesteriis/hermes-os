<script setup lang="ts">
import { computed } from 'vue'

type SectionTone = 'plain' | 'muted' | 'bordered'
type SectionPadding = 'none' | 'sm' | 'md' | 'lg'

const props = withDefaults(defineProps<{
	as?: string
	tone?: SectionTone
	padding?: SectionPadding
	class?: string
}>(), {
	as: 'section',
	tone: 'plain',
	padding: 'md'
})

const classes = computed(() => [
	'hermes-section',
	`hermes-section--${props.tone}`,
	`hermes-section--padding-${props.padding}`,
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<header v-if="$slots.header || $slots.actions" class="hermes-section-header">
			<div v-if="$slots.header" class="hermes-section-heading">
				<slot name="header" />
			</div>
			<div v-if="$slots.actions" class="hermes-section-actions">
				<slot name="actions" />
			</div>
		</header>
		<div class="hermes-section-body">
			<slot />
		</div>
		<footer v-if="$slots.footer" class="hermes-section-footer">
			<slot name="footer" />
		</footer>
	</component>
</template>
