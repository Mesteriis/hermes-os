<script setup lang="ts">
import { computed, inject } from 'vue'
import { hermesRadioGroupKey } from './RadioGroup.context'

const props = withDefaults(defineProps<{
	value: string | number
	label?: string
	disabled?: boolean
	name?: string
	modelValue?: string | number | null
	class?: string
}>(), {
	disabled: false,
	modelValue: null
})

const emit = defineEmits<{
	'update:modelValue': [value: string | number]
}>()

const group = inject(hermesRadioGroupKey, null)
const selectedValue = computed(() => group?.modelValue.value ?? props.modelValue)
const isDisabled = computed(() => props.disabled || Boolean(group?.disabled.value))
const inputName = computed(() => group?.name ?? props.name)
const checked = computed(() => selectedValue.value === props.value)
const classes = computed(() => ['hermes-radio', { 'hermes-radio--disabled': isDisabled.value }, props.class])

function handleChange(): void {
	if (isDisabled.value) return
	group?.select(props.value)
	emit('update:modelValue', props.value)
}
</script>

<template>
	<label :class="classes">
		<input
			class="hermes-radio-input"
			:checked="checked"
			:disabled="isDisabled"
			:name="inputName"
			type="radio"
			:value="value"
			@change="handleChange"
		/>
		<span><slot>{{ label }}</slot></span>
	</label>
</template>
