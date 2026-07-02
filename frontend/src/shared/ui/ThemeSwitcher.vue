<script setup lang="ts">
import { computed } from 'vue'
import type { UiThemeName, UiThemeOption } from './theme'
import { uiThemeOptions } from './theme'

const props = withDefaults(defineProps<{
	modelValue?: UiThemeName
	options?: UiThemeOption[]
	label?: string
	disabled?: boolean
	class?: string
}>(), {
	modelValue: 'light',
	label: 'Theme',
	disabled: false
})

const emit = defineEmits<{
	'update:modelValue': [value: UiThemeName]
}>()

const options = computed(() => props.options?.length ? props.options : uiThemeOptions)
const classes = computed(() => ['hermes-theme-switcher', props.class])
</script>

<template>
	<div :class="classes" role="radiogroup" :aria-label="label">
		<button
			v-for="option in options"
			:key="option.value"
			class="hermes-theme-switcher__option"
			type="button"
			role="radio"
			:aria-checked="modelValue === option.value"
			:disabled="disabled"
			:title="option.description"
			@click="emit('update:modelValue', option.value)"
		>
			<span class="hermes-theme-switcher__swatch" :data-theme-swatch="option.value" aria-hidden="true" />
			<span>{{ option.label }}</span>
		</button>
	</div>
</template>
