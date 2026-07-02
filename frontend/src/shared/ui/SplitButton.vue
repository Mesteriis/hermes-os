<script setup lang="ts">
import { computed, ref } from 'vue'
import Button from './Button.vue'
import Icon from './Icon.vue'
import IconButton from './IconButton.vue'
import type { NavigationItem } from './Navigation.types'

const props = withDefaults(defineProps<{
	label: string
	items?: NavigationItem[]
	modelValue?: string
	icon?: string
	variant?: 'default' | 'secondary' | 'outline' | 'ghost' | 'destructive'
	size?: 'sm' | 'md' | 'lg'
	disabled?: boolean
	menuLabel?: string
	class?: string
}>(), {
	items: () => [],
	modelValue: '',
	variant: 'default',
	size: 'md',
	disabled: false,
	menuLabel: 'Open action menu'
})

const emit = defineEmits<{
	click: [event: MouseEvent]
	'update:modelValue': [value: string]
	select: [item: NavigationItem]
}>()

const isOpen = ref(false)
const classes = computed(() => ['hermes-split-button', props.class])

function handlePrimaryClick(event: MouseEvent): void {
	if (props.disabled) {
		return
	}
	emit('click', event)
}

function toggleMenu(): void {
	if (!props.disabled && props.items.length > 0) {
		isOpen.value = !isOpen.value
	}
}

function selectItem(item: NavigationItem): void {
	if (item.disabled) {
		return
	}
	emit('update:modelValue', item.id)
	emit('select', item)
	isOpen.value = false
}
</script>

<template>
	<div :class="classes" @keydown.escape="isOpen = false">
		<Button
			class="hermes-split-button__main"
			:disabled="disabled"
			:icon="icon"
			:size="size"
			:variant="variant"
			@click="handlePrimaryClick"
		>
			{{ label }}
		</Button>
		<IconButton
			class="hermes-split-button__toggle"
			icon="tabler:chevron-down"
			:aria-expanded="isOpen"
			:disabled="disabled || items.length === 0"
			:label="menuLabel"
			:size="size"
			:variant="variant"
			@click="toggleMenu"
		/>
		<div v-if="isOpen" class="hermes-split-button__menu" role="menu" :aria-label="menuLabel">
			<button
				v-for="item in items"
				:key="item.id"
				class="hermes-split-button__item"
				role="menuitem"
				type="button"
				:aria-current="item.id === modelValue ? 'true' : undefined"
				:disabled="item.disabled"
				@click="selectItem(item)"
			>
				<Icon v-if="item.icon" :icon="item.icon" size="1rem" class="hermes-split-button__icon" aria-hidden="true" />
				<span>{{ item.label }}</span>
			</button>
		</div>
	</div>
</template>
