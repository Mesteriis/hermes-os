<script setup lang="ts">
import { computed, provide } from 'vue'
import { hermesRadioGroupKey } from './RadioGroup.context'

const props = withDefaults(defineProps<{
	modelValue?: string | number | null
	name?: string
	label?: string
	disabled?: boolean
	class?: string
}>(), {
	modelValue: null,
	name: 'hermes-radio-group',
	disabled: false
})

const emit = defineEmits<{
	'update:modelValue': [value: string | number]
}>()

const classes = computed(() => ['hermes-radio-group', props.class])

provide(hermesRadioGroupKey, {
	name: props.name,
	modelValue: computed(() => props.modelValue),
	disabled: computed(() => props.disabled),
	select(value: string | number): void {
		emit('update:modelValue', value)
	}
})
</script>

<template>
	<fieldset :class="classes" :disabled="disabled">
		<legend v-if="label" class="hermes-radio-group__legend">{{ label }}</legend>
		<slot />
	</fieldset>
</template>
