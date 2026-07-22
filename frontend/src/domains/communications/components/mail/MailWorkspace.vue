<script setup lang="ts">
import { ref } from 'vue'
import { AlertDialog, Dialog, Icon, RichTextEditor } from '@/shared/ui'
import type { CommunicationConversationModel } from '../communicationDomainElements'
import type {
  CommunicationAccountOption,
  ComposeFormModel,
  MailSyncStatus
} from '../../types/communications'
import '../communicationDomainElements.css'
import { useMailWorkspaceController } from '../../queries/useMailWorkspaceController'
import MailInspector from './MailInspector.vue'
import MailList from './MailList.vue'
import MailMessage from './MailMessage.vue'
import type { MailListItemModel } from './mailElements'
import type { MailInspectorModel } from './mailInspector'
import { composeAccountOptionLabel, formatComposeAttachmentSize } from './mailComposePresentation'

const props = defineProps<{
  items: readonly MailListItemModel[]
  conversation: CommunicationConversationModel
  inspector: MailInspectorModel
  hasMoreItems?: boolean
  isActionRunning?: boolean
  isImporting?: boolean
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
  'import-mail-file': [file: File]
  'attach-compose-files': [files: File[]]
  'load-more': []
  'new-message': []
  refresh: []
  'save-compose': []
  'select-action': [actionId: string]
  'select-message': [item: MailListItemModel]
  'send-compose': []
  'remove-compose-attachment': [attachmentId: string]
  'toggle-inspector': []
  'update-search-query': [query: string]
  'visible-items-change': [itemIds: string[]]
  'update-compose': [partial: Partial<ComposeFormModel>]
}>()

const composeAttachmentInput = ref<HTMLInputElement | null>(null)
const controller = useMailWorkspaceController(props, {
  closeCompose: () => emit('close-compose'),
  saveCompose: () => emit('save-compose'),
  toggleInspector: () => emit('toggle-inspector'),
  updateCompose: (partial) => emit('update-compose', partial),
  attachComposeFiles: (files) => emit('attach-compose-files', files),
})
const {
  t,
  isInspectorVisible,
  isAiComposePanelOpen,
  isContextComposePanelOpen,
  isComposeCloseConfirmOpen,
  isCcVisible,
  isBccVisible,
  isComposeDropActive,
  activeMessage,
  composeAccountOptions,
  composeAiActions,
  composeContextSections,
  composeEditorContent,
  composeAttachments,
  composeAttachmentsError,
  isComposeDialogOpen,
  composeDialogTitle,
  composeActivePanelState,
  handleToggleInspector,
  toggleComposeEdgePanel,
  showCcField,
  showBccField,
  handleComposeDialogOpenChange,
  handleComposeEscape,
  requestComposeClose,
  handleComposeCloseConfirmOpenChange,
  handleDiscardComposeDraft,
  handleSaveComposeDraftAndClose,
  handleComposeBodyHtmlChange,
  handleComposeDrop: handleDroppedFiles,
} = controller

function inputValue(event: Event): string {
  const target = event.target
  return target instanceof HTMLInputElement
    || target instanceof HTMLTextAreaElement
    || target instanceof HTMLSelectElement
    ? target.value
    : ''
}

function openComposeAttachmentPicker(): void {
  composeAttachmentInput.value?.click()
}

function handleComposeAttachmentInput(event: Event): void {
  if (!(event.target instanceof HTMLInputElement)) return
  const input = event.target
  handleDroppedFiles(input.files)
  input.value = ''
}

function handleComposeDrop(event: DragEvent): void {
  handleDroppedFiles(event.dataTransfer?.files)
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
			:is-importing="isImporting"
			:search-query="searchQuery"
			:sync-status="syncStatus"
			@compose="emit('new-message')"
			@import-mail-file="emit('import-mail-file', $event)"
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
			:title="composeDialogTitle"
			:close-label="t('Close compose')"
			:close-on-interact-outside="false"
			content-class="mail-compose-dialog"
			@update:open="handleComposeDialogOpenChange"
		>
			<template #chrome>
				<template v-if="composeForm">
					<aside
						class="compose-edge-panel compose-edge-panel--left"
						:class="{ 'is-open': isAiComposePanelOpen }"
						:aria-label="t('AI writing tools')"
						@keydown.esc.stop="handleComposeEscape"
					>
						<nav class="compose-edge-panel__rail" :aria-label="t('AI commands')">
							<button
								type="button"
								class="compose-edge-panel__toggle compose-edge-panel__rail-button"
								:aria-expanded="isAiComposePanelOpen"
								:title="isAiComposePanelOpen ? t('Hide AI') : t('Show AI')"
								@click="toggleComposeEdgePanel('ai')"
							>
								<Icon
									:icon="isAiComposePanelOpen ? 'tabler:chevron-right' : 'tabler:sparkles'"
									size="1rem"
								/>
								<span>{{ isAiComposePanelOpen ? t('Hide AI') : t('Show AI') }}</span>
							</button>
							<button
								v-for="action in composeAiActions"
								:key="action.id"
								type="button"
								class="compose-edge-panel__rail-button"
								:title="t(action.label)"
								:aria-label="t(action.label)"
								:disabled="action.disabled"
							>
								<Icon :icon="action.icon" size="1rem" />
								<span>{{ t(action.label) }}</span>
							</button>
						</nav>
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
					<aside
						class="compose-edge-panel compose-edge-panel--right"
						:class="{ 'is-open': isContextComposePanelOpen }"
						:aria-label="t('Compose context tools')"
						@keydown.esc.stop="handleComposeEscape"
					>
						<nav class="compose-edge-panel__rail" :aria-label="t('Context commands')">
							<button
								type="button"
								class="compose-edge-panel__toggle compose-edge-panel__rail-button"
								:aria-expanded="isContextComposePanelOpen"
								:title="isContextComposePanelOpen ? t('Hide context') : t('Show context')"
								@click="toggleComposeEdgePanel('context')"
							>
								<Icon
									:icon="isContextComposePanelOpen ? 'tabler:chevron-left' : 'tabler:layout-sidebar-right'"
									size="1rem"
								/>
								<span>{{ isContextComposePanelOpen ? t('Hide context') : t('Show context') }}</span>
							</button>
							<button
								v-for="section in composeContextSections"
								:key="section.id"
								type="button"
								class="compose-edge-panel__rail-button"
								:title="t(section.title)"
								:aria-label="t(section.title)"
							>
								<Icon :icon="section.icon" size="1rem" />
								<span>{{ t(section.title) }}</span>
							</button>
						</nav>
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
								<p v-for="item in section.items" :key="item">{{ t(item) }}</p>
							</section>
						</div>
					</aside>
				</template>
			</template>
			<section
				v-if="composeForm"
				class="mail-compose-stage"
					:class="[
						isAiComposePanelOpen && 'mail-compose-stage--ai-open',
						isContextComposePanelOpen && 'mail-compose-stage--context-open',
						isComposeDropActive && 'mail-compose-stage--drop-active'
					]"
				:data-active-panel="composeActivePanelState"
					@keydown.esc.stop="handleComposeEscape"
					@dragenter.prevent="isComposeDropActive = true"
					@dragover.prevent="isComposeDropActive = true"
					@dragleave.self="isComposeDropActive = false"
					@drop.prevent="handleComposeDrop"
			>
				<section class="mail-compose-panel mail-compose-card" :aria-label="composeDialogTitle">
					<div v-if="composeStatus" class="mail-compose-panel__status-row">
						<span v-if="composeStatus" class="mail-compose-panel__status">{{ composeStatus }}</span>
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
									{{ composeAccountOptionLabel(account, t) }}
								</option>
							</select>
						</label>
						<div class="mail-compose-panel__recipient-row">
							<label class="mail-compose-panel__field mail-compose-panel__field--to">
								<span>{{ t('To') }}</span>
								<input
									:value="composeForm.toText"
									type="text"
									autocomplete="email"
									@input="emit('update-compose', { toText: inputValue($event) })"
								/>
							</label>
							<div
								v-if="!isCcVisible || !isBccVisible"
								class="mail-compose-panel__recipient-actions"
								:aria-label="t('Optional recipients')"
							>
								<button
									v-if="!isCcVisible"
									type="button"
									class="mail-compose-panel__field-toggle"
									@click="showCcField"
								>
									{{ t('Cc') }}
								</button>
								<button
									v-if="!isBccVisible"
									type="button"
									class="mail-compose-panel__field-toggle"
									@click="showBccField"
								>
									{{ t('Bcc') }}
								</button>
							</div>
						</div>
						<label
							v-if="isCcVisible"
							class="mail-compose-panel__field mail-compose-panel__field--cc"
						>
							<span>{{ t('Cc') }}</span>
							<input
								:value="composeForm.ccText"
								type="text"
								autocomplete="email"
								@input="emit('update-compose', { ccText: inputValue($event) })"
							/>
						</label>
						<label
							v-if="isBccVisible"
							class="mail-compose-panel__field mail-compose-panel__field--bcc"
						>
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
							<section class="mail-compose-attachments" :aria-label="t('Attachments')">
								<div class="mail-compose-attachments__header">
									<strong>{{ t('Attachments') }}</strong>
									<button
										type="button"
										class="mail-compose-panel__field-toggle"
										:disabled="isSending"
										@click="openComposeAttachmentPicker"
									>
										<Icon icon="tabler:paperclip" size="1rem" />
										{{ t('Attach files') }}
									</button>
									<input
										ref="composeAttachmentInput"
										class="mail-compose-attachments__input"
										type="file"
										multiple
										@change="handleComposeAttachmentInput"
									/>
								</div>
								<p v-if="composeAttachments.length === 0" class="mail-compose-attachments__hint">
									{{ t('Drop files here or choose files. Files are sent only after a clean security scan.') }}
								</p>
								<ul v-else class="mail-compose-attachments__list">
									<li
										v-for="attachment in composeAttachments"
										:key="attachment.attachmentId"
										class="mail-compose-attachment"
										:class="`mail-compose-attachment--${attachment.uploadStatus}`"
									>
										<Icon
											:icon="attachment.uploadStatus === 'ready' ? 'tabler:shield-check' : 'tabler:paperclip'"
											size="1rem"
										/>
										<span class="mail-compose-attachment__name">{{ attachment.filename }}</span>
										<small>{{ formatComposeAttachmentSize(attachment.sizeBytes) }} · {{ attachment.scanStatus }}</small>
										<button
											type="button"
											:aria-label="`${t('Remove')} ${attachment.filename}`"
											:disabled="isSending"
											@click="emit('remove-compose-attachment', attachment.attachmentId)"
										>
											<Icon icon="tabler:x" size="0.9rem" />
										</button>
										<p v-if="attachment.error">{{ attachment.error }}</p>
									</li>
								</ul>
							</section>
							<div class="mail-compose-panel__body-field">
							<RichTextEditor
								class="mail-compose-panel__editor"
								:model-value="composeEditorContent"
								:label="t('Body')"
								:placeholder="t('Write email')"
								:toolbar-label="t('Mail formatting')"
								:disabled="isSending"
								@update:model-value="handleComposeBodyHtmlChange"
							/>
						</div>
					</div>
				</section>
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
							:disabled="isSending || Boolean(composeAttachmentsError)"
							:title="composeAttachmentsError"
						@click="emit('send-compose')"
					>
						<Icon :icon="isSending ? 'tabler:loader-2' : 'tabler:send'" size="1rem" />
						{{ isSending ? t('Sending') : t('Send') }}
					</button>
				</template>
			</template>
		</Dialog>
		<AlertDialog
			:open="isComposeCloseConfirmOpen"
			:title="t('Close draft?')"
			:description="t('This email has unsaved content. Save it as a draft before closing?')"
			:cancel-label="t('Keep writing')"
			:action-label="t('Close without saving')"
			tone="danger"
			content-class="mail-compose-close-confirm"
			@update:open="handleComposeCloseConfirmOpenChange"
			@cancel="handleComposeCloseConfirmOpenChange(false)"
			@action="handleDiscardComposeDraft"
		>
			<button
				type="button"
				class="mail-compose-close-confirm__save"
				:disabled="isSending"
				@click="handleSaveComposeDraftAndClose"
			>
				<Icon icon="tabler:device-floppy" size="1rem" />
				{{ t('Save draft and close') }}
			</button>
		</AlertDialog>
	</section>
</template>
