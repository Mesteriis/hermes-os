<script setup lang="ts">
import { computed } from 'vue'

type CardVariant = 'default' | 'muted' | 'raised' | 'interactive'
type CardDensity = 'compact' | 'comfortable'

const props = withDefaults(defineProps<{
	as?: string
	variant?: CardVariant
	density?: CardDensity
	selected?: boolean
	disabled?: boolean
	clip?: boolean
	class?: string
}>(), {
	as: 'article',
	variant: 'default',
	density: 'comfortable',
	selected: false,
	disabled: false,
	clip: false
})

const classes = computed(() => [
	'hermes-card',
	`hermes-card--${props.variant}`,
	`hermes-card--density-${props.density}`,
	props.selected && 'hermes-card--selected',
	props.disabled && 'hermes-card--disabled',
	props.clip && 'hermes-card--clip',
	props.class
])

const componentAttrs = computed(() => ({
	'aria-disabled': props.disabled ? 'true' : undefined,
	disabled: props.disabled && props.as === 'button' ? true : undefined
}))

function handleClick(event: MouseEvent): void {
	if (!props.disabled) {
		return
	}

	event.preventDefault()
	event.stopImmediatePropagation()
	event.stopPropagation()
}
</script>

<template>
	<component :is="as" :class="classes" v-bind="componentAttrs" @click.capture="handleClick">
		<slot />
	</component>
</template>
