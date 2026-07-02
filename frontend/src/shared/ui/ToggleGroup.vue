<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

interface ToggleGroupItem {
	value: string
	label: string
	icon?: string
	disabled?: boolean
}

const props = withDefaults(defineProps<{
	modelValue?: string | string[]
	items?: ToggleGroupItem[]
	multiple?: boolean
	ariaLabel?: string
	disabled?: boolean
	class?: string
}>(), {
	items: () => [],
	multiple: false,
	ariaLabel: 'Toggle group',
	disabled: false
})

const emit = defineEmits<{
	'update:modelValue': [value: string | string[]]
	select: [value: string]
}>()

const classes = computed(() => [
	'hermes-toggle-group',
	{ 'hermes-toggle-group--multiple': props.multiple },
	props.class
])

const selectedValues = computed(() => new Set(Array.isArray(props.modelValue)
	? props.modelValue
	: props.modelValue
		? [props.modelValue]
		: []
))

function toggleItem(item: ToggleGroupItem): void {
	if (props.disabled || item.disabled) {
		return
	}
	if (props.multiple) {
		const next = new Set(selectedValues.value)
		if (next.has(item.value)) {
			next.delete(item.value)
		} else {
			next.add(item.value)
		}
		emit('update:modelValue', Array.from(next))
	} else {
		emit('update:modelValue', item.value)
	}
	emit('select', item.value)
}
</script>

<template>
	<div :class="classes" role="group" :aria-label="ariaLabel">
		<button
			v-for="item in items"
			:key="item.value"
			class="hermes-toggle-group__item"
			type="button"
			:aria-pressed="selectedValues.has(item.value)"
			:disabled="disabled || item.disabled"
			@click="toggleItem(item)"
		>
			<Icon v-if="item.icon" :icon="item.icon" size="1rem" class="hermes-toggle-group__icon" aria-hidden="true" />
			<span>{{ item.label }}</span>
		</button>
	</div>
</template>
