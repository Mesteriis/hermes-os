<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

type CalloutTone = 'neutral' | 'info' | 'success' | 'warning' | 'danger'

const props = withDefaults(defineProps<{
	tone?: CalloutTone
	icon?: string
	class?: string
}>(), {
	tone: 'neutral'
})

const toneIcons: Record<CalloutTone, string> = {
	neutral: 'tabler:info-circle',
	info: 'tabler:info-circle',
	success: 'tabler:check-circle',
	warning: 'tabler:alert-triangle',
	danger: 'tabler:alert-circle'
}

const classes = computed(() => [
	'hermes-callout',
	`hermes-callout--${props.tone}`,
	props.class
])

const resolvedIcon = computed(() => props.icon ?? toneIcons[props.tone])
</script>

<template>
	<section :class="classes">
		<Icon
			v-if="resolvedIcon"
			:icon="resolvedIcon"
			size="1.125rem"
			class="hermes-callout-icon"
		/>
		<div class="hermes-callout-body">
			<div v-if="$slots.title" class="hermes-callout-title">
				<slot name="title" />
			</div>
			<div class="hermes-callout-content">
				<slot />
			</div>
		</div>
		<div v-if="$slots.actions" class="hermes-callout-actions">
			<slot name="actions" />
		</div>
	</section>
</template>
