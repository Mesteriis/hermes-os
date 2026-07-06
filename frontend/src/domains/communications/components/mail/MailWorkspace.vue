<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '@/platform/i18n'
import { Icon } from '@/shared/ui'
import type { CommunicationConversationModel } from '../communicationDomainElements'
import type { ComposeFormModel, MailSyncStatus } from '../../types/communications'
import '../communicationDomainElements.css'
import MailInspector from './MailInspector.vue'
import MailList from './MailList.vue'
import MailMessage from './MailMessage.vue'
import type { MailListItemModel } from './mailElements'
import type { MailInspectorModel } from './mailInspector'

const props = defineProps<{
  items: readonly MailListItemModel[]
  conversation: CommunicationConversationModel
  inspector: MailInspectorModel
  hasMoreItems?: boolean
  isActionRunning?: boolean
  isLoadingMore?: boolean
  composeOpen?: boolean
  composeForm?: ComposeFormModel
  composeStatus?: string
  composeError?: string
  isSending?: boolean
  searchQuery?: string
  syncStatus?: MailSyncStatus | null
}>()

const emit = defineEmits<{
  'close-compose': []
  'load-more': []
  'new-message': []
  refresh: []
  'save-compose': []
  'select-action': [actionId: string]
  'select-message': [item: MailListItemModel]
  'send-compose': []
  'toggle-inspector': []
  'update-search-query': [query: string]
  'update-compose': [partial: Partial<ComposeFormModel>]
}>()

const { t } = useI18n()
const isInspectorVisible = ref(true)
const activeMessage = computed(() => props.conversation.messages[props.conversation.messages.length - 1])
const composeTitle = computed(() => {
  switch (props.composeForm?.mode) {
    case 'reply': return t('Reply')
    case 'forward': return t('Forward')
    default: return t('Compose')
  }
})

function handleToggleInspector(): void {
  isInspectorVisible.value = !isInspectorVisible.value
  emit('toggle-inspector')
}

function inputValue(event: Event): string {
  return (event.target as HTMLInputElement | HTMLTextAreaElement).value
}
</script>

<template>
	<section
		:class="[
			'communication-workspace-shell communication-workspace-shell--mail',
			!isInspectorVisible && 'communication-workspace-shell--mail-inspector-hidden'
		]"
	>
		<MailList
			:items="items"
			:has-more-items="hasMoreItems"
			:is-loading-more="isLoadingMore"
			:search-query="searchQuery"
			:sync-status="syncStatus"
			@compose="emit('new-message')"
			@load-more="emit('load-more')"
			@refresh="emit('refresh')"
			@select-item="emit('select-message', $event)"
			@update-search-query="emit('update-search-query', $event)"
		/>
		<section class="communication-mail-workspace-reader" :aria-label="t('Open message')">
			<MailMessage
				v-if="activeMessage"
				:message="activeMessage"
				:fallback-subject="conversation.title"
				:inspector-visible="isInspectorVisible"
				:is-action-running="isActionRunning"
				@select-action="emit('select-action', $event)"
				@toggle-inspector="handleToggleInspector"
			/>
			<p v-else class="communication-mail-workspace-reader__empty">{{ t('No message selected.') }}</p>
			<section v-if="composeOpen && composeForm" class="mail-compose-panel" :aria-label="composeTitle">
				<header class="mail-compose-panel__header">
					<div class="mail-compose-panel__title-group">
						<strong class="mail-compose-panel__title">{{ composeTitle }}</strong>
						<span v-if="composeStatus" class="mail-compose-panel__status">{{ composeStatus }}</span>
						<span v-if="composeError" class="mail-compose-panel__status mail-compose-panel__status--error">{{ composeError }}</span>
					</div>
					<button
						type="button"
						class="mail-compose-panel__close"
						:aria-label="t('Close compose')"
						:title="t('Close compose')"
						@click="emit('close-compose')"
					>
						<Icon icon="tabler:x" size="1rem" />
					</button>
				</header>
				<div class="mail-compose-panel__fields">
					<label class="mail-compose-panel__field">
						<span>{{ t('To') }}</span>
						<input
							:value="composeForm.toText"
							type="text"
							autocomplete="email"
							@input="emit('update-compose', { toText: inputValue($event) })"
						/>
					</label>
					<label class="mail-compose-panel__field">
						<span>{{ t('Cc') }}</span>
						<input
							:value="composeForm.ccText"
							type="text"
							autocomplete="email"
							@input="emit('update-compose', { ccText: inputValue($event) })"
						/>
					</label>
					<label class="mail-compose-panel__field">
						<span>{{ t('Bcc') }}</span>
						<input
							:value="composeForm.bccText"
							type="text"
							autocomplete="email"
							@input="emit('update-compose', { bccText: inputValue($event) })"
						/>
					</label>
					<label class="mail-compose-panel__field mail-compose-panel__field--subject">
						<span>{{ t('Subject') }}</span>
						<input
							:value="composeForm.subject"
							type="text"
							@input="emit('update-compose', { subject: inputValue($event) })"
						/>
					</label>
					<label class="mail-compose-panel__body-field">
						<span>{{ t('Body') }}</span>
						<textarea
							:value="composeForm.body"
							rows="6"
							@input="emit('update-compose', { body: inputValue($event), bodyFormat: 'plain', bodyHtml: null })"
						/>
					</label>
				</div>
				<footer class="mail-compose-panel__actions">
					<button
						type="button"
						class="mail-compose-panel__button mail-compose-panel__button--secondary"
						:disabled="isSending"
						@click="emit('save-compose')"
					>
						<Icon icon="tabler:device-floppy" size="1rem" />
						{{ t('Save draft') }}
					</button>
					<button
						type="button"
						class="mail-compose-panel__button mail-compose-panel__button--primary"
						:disabled="isSending"
						@click="emit('send-compose')"
					>
						<Icon :icon="isSending ? 'tabler:loader-2' : 'tabler:send'" size="1rem" />
						{{ isSending ? t('Sending') : t('Send') }}
					</button>
				</footer>
			</section>
		</section>
		<MailInspector v-if="isInspectorVisible" :model="inspector" />
	</section>
</template>
