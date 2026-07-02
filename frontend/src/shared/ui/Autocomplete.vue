<script setup lang="ts">
import { computed, ref } from 'vue'

interface AutocompleteOption {
	value: string
	label: string
	disabled?: boolean
}

const MAX_AUTOCOMPLETE_OPTIONS = 8

const props = withDefaults(defineProps<{
	id?: string
	modelValue?: string
	options?: AutocompleteOption[]
	placeholder?: string
	ariaLabel?: string
	noResultsLabel?: string
	disabled?: boolean
	readonly?: boolean
	class?: string
}>(), {
	modelValue: '',
	options: () => [],
	placeholder: '',
	noResultsLabel: 'No results',
	disabled: false,
	readonly: false
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	select: [option: AutocompleteOption]
}>()

const isOpen = ref(false)
const activeIndex = ref(0)

const classes = computed(() => ['hermes-autocomplete', props.class])
const listId = computed(() => `${props.id ?? 'hermes-autocomplete'}-listbox`)
const filteredOptions = computed(() => {
	const query = props.modelValue.trim().toLocaleLowerCase()
	const options = query
		? props.options.filter((option) => option.label.toLocaleLowerCase().includes(query) || option.value.toLocaleLowerCase().includes(query))
		: props.options
	return options.filter((option) => !option.disabled).slice(0, MAX_AUTOCOMPLETE_OPTIONS)
})
const activeDescendant = computed(() => {
	if (!isOpen.value || filteredOptions.value.length === 0) {
		return undefined
	}
	return optionId(activeIndex.value)
})

function optionId(index: number): string {
	return `${listId.value}-${index}`
}

function handleInput(event: Event): void {
	const target = event.target as HTMLInputElement
	emit('update:modelValue', target.value)
	isOpen.value = true
	activeIndex.value = 0
}

function selectOption(option: AutocompleteOption): void {
	emit('update:modelValue', option.value)
	emit('select', option)
	isOpen.value = false
}

function handleKeydown(event: KeyboardEvent): void {
	if (!isOpen.value && ['ArrowDown', 'ArrowUp'].includes(event.key)) {
		isOpen.value = true
	}
	if (event.key === 'Escape') {
		isOpen.value = false
		return
	}
	if (filteredOptions.value.length === 0) {
		return
	}
	if (event.key === 'ArrowDown') {
		event.preventDefault()
		activeIndex.value = Math.min(activeIndex.value + 1, filteredOptions.value.length - 1)
	}
	if (event.key === 'ArrowUp') {
		event.preventDefault()
		activeIndex.value = Math.max(activeIndex.value - 1, 0)
	}
	if (event.key === 'Home') {
		event.preventDefault()
		activeIndex.value = 0
	}
	if (event.key === 'End') {
		event.preventDefault()
		activeIndex.value = filteredOptions.value.length - 1
	}
	if (event.key === 'Enter' && isOpen.value) {
		event.preventDefault()
		selectOption(filteredOptions.value[activeIndex.value])
	}
}

function handleFocusout(event: FocusEvent): void {
	const currentTarget = event.currentTarget as HTMLElement
	const nextTarget = event.relatedTarget
	if (!(nextTarget instanceof Node) || !currentTarget.contains(nextTarget)) {
		isOpen.value = false
	}
}
</script>

<template>
	<div :class="classes" @focusout="handleFocusout">
		<input
			class="hermes-native-control"
			:aria-activedescendant="activeDescendant"
			:aria-autocomplete="'list'"
			:aria-controls="listId"
			:aria-expanded="isOpen"
			:aria-label="ariaLabel"
			:disabled="disabled"
			:id="id"
			:placeholder="placeholder"
			:readonly="readonly"
			role="combobox"
			:type="'text'"
			:value="modelValue"
			@focus="isOpen = true"
			@input="handleInput"
			@keydown="handleKeydown"
		/>
		<ul v-if="isOpen" :id="listId" class="hermes-autocomplete__listbox" role="listbox">
			<li
				v-for="(option, index) in filteredOptions"
				:id="optionId(index)"
				:key="option.value"
				class="hermes-autocomplete__option"
				:aria-selected="index === activeIndex"
				role="option"
				tabindex="-1"
				@mousedown.prevent="selectOption(option)"
			>
				{{ option.label }}
			</li>
			<li v-if="filteredOptions.length === 0" class="hermes-autocomplete__empty" role="presentation">
				{{ noResultsLabel }}
			</li>
		</ul>
	</div>
</template>
