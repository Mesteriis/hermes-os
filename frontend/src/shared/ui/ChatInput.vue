<script setup lang="ts">
import { computed } from 'vue'
import Button from './Button.vue'
import IconButton from './IconButton.vue'

const props = withDefaults(defineProps<{
	modelValue?: string
	id?: string
	label?: string
	placeholder?: string
	helper?: string
	sendLabel?: string
	attachLabel?: string
	disabled?: boolean
	loading?: boolean
	rows?: number
	maxLength?: number
	showAttach?: boolean
	class?: string
}>(), {
	modelValue: '',
	placeholder: 'Write a message',
	sendLabel: 'Send',
	attachLabel: 'Attach',
	disabled: false,
	loading: false,
	rows: 3,
	showAttach: true
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	submit: [value: string]
	attach: []
}>()

const classes = computed(() => [
	'hermes-chat-input',
	{
		'hermes-chat-input--disabled': props.disabled,
		'hermes-chat-input--loading': props.loading
	},
	props.class
])
const canSubmit = computed(() => props.modelValue.trim().length > 0 && !props.disabled && !props.loading)
const remaining = computed(() => {
	if (props.maxLength == null) {
		return undefined
	}
	return props.maxLength - props.modelValue.length
})

function handleInput(event: Event): void {
	const target = event.target as HTMLTextAreaElement
	emit('update:modelValue', target.value)
}

function submit(): void {
	if (canSubmit.value) {
		emit('submit', props.modelValue)
	}
}
</script>

<template>
	<form :class="classes" @submit.prevent="submit">
		<label v-if="label" class="hermes-chat-input__label" :for="id">{{ label }}</label>
		<div class="hermes-chat-input__surface">
			<textarea
				:id="id"
				class="hermes-chat-input__textarea"
				:disabled="disabled || loading"
				:maxlength="maxLength"
				:placeholder="placeholder"
				:rows="rows"
				:value="modelValue"
				@input="handleInput"
				@keydown.ctrl.enter.prevent="submit"
				@keydown.meta.enter.prevent="submit"
			/>
			<div class="hermes-chat-input__actions">
				<IconButton
					v-if="showAttach"
					icon="tabler:paperclip"
					:label="attachLabel"
					size="sm"
					variant="ghost"
					:disabled="disabled || loading"
					@click="emit('attach')"
				/>
				<slot name="toolbar" />
				<Button type="submit" size="sm" :loading="loading" :disabled="!canSubmit">
					{{ sendLabel }}
				</Button>
			</div>
		</div>
		<div v-if="helper || remaining !== undefined" class="hermes-chat-input__meta">
			<span v-if="helper">{{ helper }}</span>
			<span v-if="remaining !== undefined">{{ remaining }}</span>
		</div>
	</form>
</template>
