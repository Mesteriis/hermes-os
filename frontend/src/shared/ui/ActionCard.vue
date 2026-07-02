<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
	as?: string
	title: string
	description?: string
	icon?: string
	selected?: boolean
	disabled?: boolean
	class?: string
}>(), {
	as: 'button',
	selected: false,
	disabled: false
})

const emit = defineEmits<{
	click: [event: MouseEvent]
}>()

const nativeInteractiveTags = new Set(['a', 'input', 'select', 'textarea', 'summary'])

const classes = computed(() => [
	'hermes-action-card',
	props.selected && 'hermes-action-card--selected',
	props.disabled && 'hermes-action-card--disabled',
	props.class
])

const isButton = computed(() => props.as === 'button')
const isNativeInteractive = computed(() => nativeInteractiveTags.has(props.as))

const componentAttrs = computed(() => ({
	type: isButton.value ? 'button' : undefined,
	disabled: isButton.value && props.disabled ? true : undefined,
	role: !isButton.value && !isNativeInteractive.value ? 'button' : undefined,
	tabindex: !isButton.value ? (props.disabled ? -1 : 0) : undefined,
	'aria-disabled': !isButton.value && props.disabled ? 'true' : undefined,
	'aria-pressed': isButton.value ? props.selected : undefined
}))

function suppressAction(event: MouseEvent | KeyboardEvent): void {
	event.preventDefault()
	event.stopImmediatePropagation()
	event.stopPropagation()
}

function handleClick(event: MouseEvent): void {
	if (props.disabled) {
		suppressAction(event)
		return
	}

	emit('click', event)
}

function handleKeydown(event: KeyboardEvent): void {
	if (isButton.value || (event.key !== 'Enter' && event.key !== ' ')) {
		return
	}

	if (props.disabled) {
		suppressAction(event)
		return
	}

	event.preventDefault()

	if (event.currentTarget instanceof HTMLElement) {
		event.currentTarget.click()
	}
}
</script>

<template>
	<component
		:is="as"
		:class="classes"
		v-bind="componentAttrs"
		@click.capture="handleClick"
		@keydown="handleKeydown"
	>
		<Icon
			v-if="icon"
			:icon="icon"
			size="1.25rem"
			class="hermes-action-card-icon"
		/>
		<span class="hermes-action-card-body">
			<span class="hermes-action-card-title">{{ title }}</span>
			<span v-if="description" class="hermes-action-card-description">{{ description }}</span>
		</span>
	</component>
</template>
