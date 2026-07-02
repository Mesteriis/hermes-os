<script setup lang="ts">
import {
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogOverlay,
	DialogPortal,
	DialogRoot,
	DialogTitle,
	DialogTrigger
} from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
	open?: boolean
	title?: string
	description?: string
	closeLabel?: string
	side?: 'left' | 'right' | 'bottom'
	size?: 'compact' | 'default' | 'wide'
	class?: string
	contentClass?: string
}>(), {
	open: false,
	closeLabel: 'Close drawer',
	side: 'bottom',
	size: 'default'
})

const emit = defineEmits<{
	'update:open': [value: boolean]
}>()

const contentClasses = computed(() => [
	'hermes-drawer-content',
	`hermes-drawer--${props.side}`,
	`hermes-drawer--${props.size}`,
	props.contentClass
])
</script>

<template>
	<DialogRoot :open="open" @update:open="(value) => emit('update:open', value)">
		<DialogTrigger v-if="$slots.trigger" as-child>
			<slot name="trigger" />
		</DialogTrigger>
		<DialogPortal>
			<DialogOverlay class="hermes-drawer-overlay">
				<DialogContent :class="contentClasses">
					<div class="hermes-drawer-handle" aria-hidden="true" />
					<header class="hermes-drawer-header">
						<DialogTitle v-if="title" class="hermes-drawer-title">{{ title }}</DialogTitle>
						<DialogDescription v-if="description" class="hermes-drawer-description">
							{{ description }}
						</DialogDescription>
						<slot name="header" />
					</header>
					<div class="hermes-drawer-body">
						<slot />
					</div>
					<footer v-if="$slots.footer" class="hermes-drawer-footer">
						<slot name="footer" />
					</footer>
					<DialogClose class="hermes-drawer-close" as-child>
						<button class="hermes-drawer-close-btn" type="button" :aria-label="closeLabel">
							<Icon icon="tabler:x" size="1.125rem" />
						</button>
					</DialogClose>
				</DialogContent>
			</DialogOverlay>
		</DialogPortal>
	</DialogRoot>
</template>
