<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '@/platform/i18n'
import { Dialog, Icon, RichTextEditor } from '@/shared/ui'
import type { CommunicationConversationModel } from '../communicationDomainElements'
import type {
  CommunicationAccountOption,
  ComposeFormModel,
  MailSyncStatus
} from '../../types/communications'
import { htmlToComposePlainText, plainTextToComposeHtml } from '../richComposeHtml'
import '../communicationDomainElements.css'
import MailInspector from './MailInspector.vue'
import MailList from './MailList.vue'
import MailMessage from './MailMessage.vue'
import type { MailListItemModel } from './mailElements'
import {
  composeAccountOptionSignature,
  composeAiPanelActions,
  composeContextPanelSections,
  sendCapableComposeAccounts,
  type ComposeEdgePanelId
} from './mailComposeOptions'
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
  composeAccountOptions?: readonly CommunicationAccountOption[]
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
  'visible-items-change': [itemIds: string[]]
  'update-compose': [partial: Partial<ComposeFormModel>]
}>()

const { t } = useI18n()
const isInspectorVisible = ref(true)
const activeComposePanel = ref<ComposeEdgePanelId | null>(null)
const activeMessage = computed(() => props.conversation.messages[props.conversation.messages.length - 1])
const composeAccountOptions = computed(() => props.composeAccountOptions ?? [])
const composeSendAccountOptions = computed(() =>
  sendCapableComposeAccounts(composeAccountOptions.value)
)
const composeAiActions = computed(() => composeAiPanelActions())
const composeContextSections = computed(() => composeContextPanelSections(composeAccountOptions.value))
const composeAccountOptionKey = computed(() =>
  composeAccountOptionSignature(composeAccountOptions.value)
)
const composeEditorHtml = computed(() => {
  const form = props.composeForm
  if (!form) return '<p></p>'
  return form.bodyHtml?.trim() ? form.bodyHtml : plainTextToComposeHtml(form.body)
})
const isComposeDialogOpen = computed(() => Boolean(props.composeOpen && props.composeForm))
const composeTitle = computed(() => {
  switch (props.composeForm?.mode) {
    case 'reply': return t('Reply')
    case 'forward': return t('Forward')
    default: return t('Compose')
  }
})

watch(
  () => ({
    isOpen: Boolean(props.composeOpen),
    accountId: props.composeForm?.accountId ?? '',
    optionKey: composeAccountOptionKey.value
  }),
  ({ isOpen, accountId }) => {
    if (!isOpen || composeSendAccountOptions.value.length === 0) return
    const normalizedAccountId = accountId.trim()
    if (
      normalizedAccountId &&
      composeSendAccountOptions.value.some((account) => account.account_id === normalizedAccountId)
    ) {
      return
    }
    emit('update-compose', { accountId: composeSendAccountOptions.value[0].account_id })
  },
  { immediate: true }
)

function handleToggleInspector(): void {
  isInspectorVisible.value = !isInspectorVisible.value
  emit('toggle-inspector')
}

function handleComposeDialogOpenChange(open: boolean): void {
  if (!open) {
    closeComposeEdgePanels()
    emit('close-compose')
  }
}

function toggleComposeEdgePanel(panelId: ComposeEdgePanelId): void {
  activeComposePanel.value = activeComposePanel.value === panelId ? null : panelId
}

function closeComposeEdgePanels(): void {
  activeComposePanel.value = null
}

function handleComposeEscape(): void {
  if (activeComposePanel.value) {
    closeComposeEdgePanels()
    return
  }
  emit('close-compose')
}

function inputValue(event: Event): string {
  return (event.target as HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement).value
}

function handleComposeBodyHtmlChange(bodyHtml: string): void {
  emit('update-compose', {
    body: htmlToComposePlainText(bodyHtml),
    bodyHtml,
    bodyFormat: 'html'
  })
}

function composeAccountOptionLabel(account: CommunicationAccountOption): string {
  const label = account.email && account.email !== account.label
    ? `${account.label} · ${account.email}`
    : account.label
  return account.can_send ? label : `${label} · ${t('Read only')}`
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
			@visible-items-change="emit('visible-items-change', $event)"
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
		</section>
		<MailInspector v-if="isInspectorVisible" :model="inspector" />
		<Dialog
			:open="isComposeDialogOpen"
			:title="composeTitle"
			:close-label="t('Close compose')"
			content-class="mail-compose-dialog"
			@update:open="handleComposeDialogOpenChange"
		>
			<section
				v-if="composeForm"
				class="mail-compose-stage"
				:class="[
					activeComposePanel === 'ai' && 'mail-compose-stage--ai-open',
					activeComposePanel === 'context' && 'mail-compose-stage--context-open'
				]"
				:data-active-panel="activeComposePanel ?? 'none'"
				@keydown.esc.stop="handleComposeEscape"
			>
				<aside
					class="compose-edge-panel compose-edge-panel--left"
					:class="{ 'is-open': activeComposePanel === 'ai' }"
					:aria-label="t('AI writing tools')"
				>
					<button
						type="button"
						class="compose-edge-panel__handle"
						:aria-expanded="activeComposePanel === 'ai'"
						:title="t('AI writing tools')"
						@click="toggleComposeEdgePanel('ai')"
					>
						<Icon icon="tabler:sparkles" size="1rem" />
						<span>{{ t('AI') }}</span>
					</button>
					<div class="compose-edge-panel__surface">
						<button
							v-for="action in composeAiActions"
							:key="action.id"
							type="button"
							class="compose-edge-panel__action"
							:title="t(action.description)"
							:disabled="action.disabled"
						>
							<Icon :icon="action.icon" size="1rem" />
							<span>{{ t(action.label) }}</span>
							<small>{{ t(action.description) }}</small>
						</button>
					</div>
				</aside>
				<section class="mail-compose-panel mail-compose-card" :aria-label="composeTitle">
					<div
						v-if="composeStatus || composeError"
						class="mail-compose-panel__status-row"
					>
						<span v-if="composeStatus" class="mail-compose-panel__status">{{ composeStatus }}</span>
						<span v-if="composeError" class="mail-compose-panel__status mail-compose-panel__status--error">
							{{ composeError }}
						</span>
					</div>
					<div class="mail-compose-panel__fields">
						<label class="mail-compose-panel__field mail-compose-panel__field--from">
							<span>{{ t('From') }}</span>
							<select
								:value="composeForm.accountId"
								:disabled="isSending || composeAccountOptions.length === 0"
								@change="emit('update-compose', { accountId: inputValue($event) })"
							>
								<option value="" disabled>{{ t('Select sender account') }}</option>
								<option
									v-for="account in composeAccountOptions"
									:key="account.account_id"
									:value="account.account_id"
									:disabled="!account.can_send"
								>
									{{ composeAccountOptionLabel(account) }}
								</option>
							</select>
						</label>
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
						<div class="mail-compose-panel__body-field">
							<RichTextEditor
								class="mail-compose-panel__editor"
								:model-value="composeEditorHtml"
								:label="t('Body')"
								:placeholder="t('Write email')"
								:toolbar-label="t('Mail formatting')"
								:disabled="isSending"
								@update:model-value="handleComposeBodyHtmlChange"
							/>
						</div>
					</div>
				</section>
				<aside
					class="compose-edge-panel compose-edge-panel--right"
					:class="{ 'is-open': activeComposePanel === 'context' }"
					:aria-label="t('Compose context tools')"
				>
					<button
						type="button"
						class="compose-edge-panel__handle"
						:aria-expanded="activeComposePanel === 'context'"
						:title="t('Compose context tools')"
						@click="toggleComposeEdgePanel('context')"
					>
						<Icon icon="tabler:layout-sidebar-right" size="1rem" />
						<span>{{ t('Context') }}</span>
					</button>
					<div class="compose-edge-panel__surface">
						<section
							v-for="section in composeContextSections"
							:key="section.id"
							class="compose-edge-panel__section"
						>
							<h3>
								<Icon :icon="section.icon" size="1rem" />
								<span>{{ t(section.title) }}</span>
							</h3>
							<p v-for="item in section.items" :key="item">{{ item }}</p>
						</section>
					</div>
				</aside>
			</section>
			<template #footer>
				<template v-if="composeForm">
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
				</template>
			</template>
		</Dialog>
	</section>
</template>
