<script setup lang="ts">
import {
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogOverlay,
	AlertDialogPortal,
	AlertDialogRoot,
	AlertDialogTitle,
	AlertDialogTrigger
} from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	open?: boolean
	title?: string
	description?: string
	cancelLabel?: string
	actionLabel?: string
	tone?: 'default' | 'danger'
	class?: string
	contentClass?: string
}>(), {
	open: false,
	tone: 'danger'
})

const emit = defineEmits<{
	'update:open': [value: boolean]
	action: []
	cancel: []
}>()

const contentClasses = computed(() => [
	'hermes-alert-dialog-content',
	`hermes-alert-dialog-content--${props.tone}`,
	props.contentClass
])
</script>

<template>
	<AlertDialogRoot :open="open" @update:open="(value) => emit('update:open', value)">
		<AlertDialogTrigger v-if="$slots.trigger" as-child>
			<slot name="trigger" />
		</AlertDialogTrigger>
		<AlertDialogPortal>
			<AlertDialogOverlay class="hermes-alert-dialog-overlay">
				<AlertDialogContent :class="contentClasses">
					<div class="hermes-alert-dialog-header">
						<AlertDialogTitle v-if="title" class="hermes-alert-dialog-title">{{ title }}</AlertDialogTitle>
						<AlertDialogDescription v-if="description" class="hermes-alert-dialog-description">
							{{ description }}
						</AlertDialogDescription>
						<slot name="header" />
					</div>
					<div v-if="$slots.default" class="hermes-alert-dialog-body">
						<slot />
					</div>
					<div class="hermes-alert-dialog-footer">
						<AlertDialogCancel as-child @click="emit('cancel')">
							<button class="hermes-alert-dialog-cancel">
								<slot name="cancel">{{ cancelLabel }}</slot>
							</button>
						</AlertDialogCancel>
						<AlertDialogAction as-child @click="emit('action')">
							<button :class="['hermes-alert-dialog-action', `hermes-alert-dialog-action--${tone}`]">
								<slot name="action">{{ actionLabel }}</slot>
							</button>
						</AlertDialogAction>
					</div>
				</AlertDialogContent>
			</AlertDialogOverlay>
		</AlertDialogPortal>
	</AlertDialogRoot>
</template>
