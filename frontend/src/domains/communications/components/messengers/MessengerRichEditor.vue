<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { Component } from 'vue'
import type { MessengerConversationModel } from './messengerElements'
import { messengerComposerDraftHtml, type MessengerComposerCapability } from './messengerComposer'
import SignalMessengerRichEditor from './SignalMessengerRichEditor.vue'
import TelegramMessengerRichEditor from './TelegramMessengerRichEditor.vue'
import WhatsAppMessengerRichEditor from './WhatsAppMessengerRichEditor.vue'

const props = defineProps<{
  conversation: MessengerConversationModel
}>()

const emit = defineEmits<{
  'select-capability': [capability: MessengerComposerCapability]
  submit: [value: string]
}>()

const draftHtml = ref(messengerComposerDraftHtml(props.conversation.draftPreview))

const editorComponent = computed<Component>(() => {
  if (props.conversation.channelKind === 'telegram') return TelegramMessengerRichEditor
  if (props.conversation.channelKind === 'whatsapp') return WhatsAppMessengerRichEditor
  return SignalMessengerRichEditor
})

watch(
  () => [props.conversation.id, props.conversation.channelKind, props.conversation.draftPreview] as const,
  () => {
    draftHtml.value = messengerComposerDraftHtml(props.conversation.draftPreview)
  }
)
</script>

<template>
	<component
		:is="editorComponent"
		v-model="draftHtml"
		:conversation="conversation"
		@select-capability="emit('select-capability', $event)"
		@submit="emit('submit', $event)"
	/>
</template>
