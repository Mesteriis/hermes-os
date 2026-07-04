<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '@/platform/i18n'
import {
  Button,
  ButtonGroup,
  DropdownMenu,
  DropdownMenuItem,
  RichTextEditor,
  Tooltip,
  type RichTextEditorToolbarAction
} from '@/shared/ui'
import '../communicationDomainElements.css'
import { localizedMessengerRichTextActions, type MessengerComposerCapability, type MessengerComposerPreset } from './messengerComposer'

const props = defineProps<{
  modelValue: string
  preset: MessengerComposerPreset
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'select-capability': [capability: MessengerComposerCapability]
  submit: [value: string]
}>()

const { t } = useI18n()

const localizedRichTextActions = computed<RichTextEditorToolbarAction[]>(() =>
  localizedMessengerRichTextActions(props.preset.richTextActions, t)
)

function handleCapabilitySelect(capability: MessengerComposerCapability): void {
  emit('select-capability', capability)
}
</script>

<template>
	<section class="messenger-provider-rich-editor" :aria-label="t('Messenger rich editor')">
		<div class="messenger-provider-rich-editor__editor-shell">
			<RichTextEditor
				class="messenger-provider-rich-editor__editor"
				:model-value="modelValue"
				:actions="localizedRichTextActions"
				:placeholder="t(preset.placeholder)"
				:toolbar-label="t(preset.toolbarLabel)"
				:output-label="t('Draft format')"
				@update:model-value="emit('update:modelValue', $event)"
			>
				<template #toolbar-end>
					<div class="messenger-provider-rich-editor__toolbar-actions">
						<ButtonGroup
							:aria-label="t('Messenger composer actions')"
							class="messenger-provider-rich-editor__button-group"
						>
							<Tooltip
								v-for="capability in preset.primaryActions"
								:key="capability.id"
								:content="t(capability.label)"
							>
								<template #trigger>
									<Button
										class="messenger-provider-rich-editor__capability-button hermes-icon-button"
										variant="outline"
										size="sm"
										:icon="capability.icon"
										:aria-label="t(capability.label)"
										:title="t(capability.label)"
										@click="handleCapabilitySelect(capability)"
									/>
								</template>
							</Tooltip>
						</ButtonGroup>

						<DropdownMenu align="end" :side-offset="8" class="messenger-provider-rich-editor__insert-menu">
							<template #trigger>
								<Button
									class="messenger-provider-rich-editor__capability-button hermes-icon-button"
									variant="outline"
									size="sm"
									icon="tabler:plus"
									:aria-label="t('More messenger tools')"
									:title="t('More messenger tools')"
								/>
							</template>
							<div class="messenger-provider-rich-editor__insert-list" :aria-label="t('More messenger tools')">
								<DropdownMenuItem
									v-for="capability in preset.insertActions"
									:key="capability.id"
									:icon="capability.icon"
									@click="handleCapabilitySelect(capability)"
								>
									{{ t(capability.label) }}
								</DropdownMenuItem>
							</div>
						</DropdownMenu>
					</div>
				</template>
			</RichTextEditor>
			<Tooltip :content="t('Send')">
				<template #trigger>
					<Button
						class="messenger-provider-rich-editor__send-button hermes-icon-button"
						size="sm"
						icon="tabler:send"
						:aria-label="t('Send')"
						:title="t('Send')"
						@click="emit('submit', modelValue)"
					/>
				</template>
			</Tooltip>
		</div>
	</section>
</template>
