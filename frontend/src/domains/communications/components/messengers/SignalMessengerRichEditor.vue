<script setup lang="ts">
import { computed } from 'vue'
import type { MessengerConversationModel } from './messengerElements'
import MessengerProviderRichEditor from './MessengerProviderRichEditor.vue'
import { signalMessengerComposerPreset, type MessengerComposerCapability } from './messengerComposer'

const props = defineProps<{
  modelValue: string
  conversation: MessengerConversationModel
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'select-capability': [capability: MessengerComposerCapability]
  submit: [value: string]
}>()

const preset = computed(() => signalMessengerComposerPreset(props.conversation))
</script>

<template>
	<MessengerProviderRichEditor
		:model-value="modelValue"
		:preset="preset"
		@select-capability="emit('select-capability', $event)"
		@submit="emit('submit', $event)"
		@update:model-value="emit('update:modelValue', $event)"
	/>
</template>
