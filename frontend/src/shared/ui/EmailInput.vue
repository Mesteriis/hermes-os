<script setup lang="ts">
import Input from './primitives/Input.vue'

const props = withDefaults(defineProps<{
	modelValue?: string
	id?: string
	placeholder?: string
	ariaLabel?: string
	disabled?: boolean
	readonly?: boolean
	error?: string
	class?: string
}>(), {
	modelValue: '',
	placeholder: '',
	disabled: false,
	readonly: false
})

defineEmits<{
	'update:modelValue': [value: string]
}>()
</script>

<template>
	<Input
		:aria-label="ariaLabel"
		:class="props.class"
		:disabled="disabled"
		:error="error"
		:id="id"
		:model-value="modelValue"
		:placeholder="placeholder"
		:readonly="readonly"
		type="email"
		@update:model-value="$emit('update:modelValue', $event)"
	/>
</template>
