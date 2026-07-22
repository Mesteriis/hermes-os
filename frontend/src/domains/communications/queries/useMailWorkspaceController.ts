import { computed, ref, watch } from 'vue'
import { useI18n } from '@/platform/i18n'
import type { CommunicationConversationModel } from '../components/communicationDomainElements'
import type {
  CommunicationAccountOption,
  ComposeFormModel,
} from '../types/communications'
import { composeAttachmentSendError } from '../forms/composeAttachmentUpload'
import { communicationConversationActiveMessage } from '../components/communicationConversationPresentation'
import {
  composeAccountOptionSignature,
  composeAiPanelActions,
  composeContextPanelSections,
  sendCapableComposeAccounts,
  type ComposeEdgePanelId,
} from '../components/mail/mailComposeOptions'
import {
  composeBodyHtmlToPlainText,
  composeEditorHtml,
  composeFormHasTypedContent,
  composePanelState,
  composeTitle,
} from '../components/mail/mailComposePresentation'

type MailWorkspaceProps = {
  conversation: CommunicationConversationModel
  composeOpen?: boolean
  composeForm?: ComposeFormModel
  composeAccountOptions?: readonly CommunicationAccountOption[]
}

export interface MailWorkspaceControllerActions {
  closeCompose(): void
  saveCompose(): void
  toggleInspector(): void
  updateCompose(partial: Partial<ComposeFormModel>): void
  attachComposeFiles(files: File[]): void
}

export function useMailWorkspaceController(
  props: Readonly<MailWorkspaceProps>,
  actions: MailWorkspaceControllerActions,
) {
  const { t } = useI18n()
  const isInspectorVisible = ref(true)
  const isAiComposePanelOpen = ref(false)
  const isContextComposePanelOpen = ref(false)
  const isComposeCloseConfirmOpen = ref(false)
  const isCcVisible = ref(false)
  const isBccVisible = ref(false)
  const isComposeDropActive = ref(false)
  const activeMessage = computed(() => communicationConversationActiveMessage(props.conversation.messages))
  const composeAccountOptions = computed(() => props.composeAccountOptions ?? [])
  const composeSendAccountOptions = computed(() => sendCapableComposeAccounts(composeAccountOptions.value))
  const composeAiActions = computed(() => composeAiPanelActions())
  const composeContextSections = computed(() => composeContextPanelSections(composeAccountOptions.value))
  const composeAccountOptionKey = computed(() => composeAccountOptionSignature(composeAccountOptions.value))
  const composeEditorContent = computed(() => composeEditorHtml(props.composeForm))
  const composeAttachments = computed(() => props.composeForm?.attachments ?? [])
  const composeAttachmentsError = computed(() => composeAttachmentSendError(composeAttachments.value))
  const isComposeDialogOpen = computed(() => Boolean(props.composeOpen && props.composeForm))
  const composeDialogTitle = computed(() => composeTitle(props.composeForm?.mode, t))
  const composeActivePanelState = computed(() => composePanelState(
    isAiComposePanelOpen.value,
    isContextComposePanelOpen.value,
  ))

  watch(
    () => ({
      isOpen: Boolean(props.composeOpen),
      accountId: props.composeForm?.accountId ?? '',
      optionKey: composeAccountOptionKey.value,
    }),
    ({ isOpen, accountId }) => {
      if (!isOpen || composeSendAccountOptions.value.length === 0) return
      const normalizedAccountId = accountId.trim()
      if (normalizedAccountId && composeSendAccountOptions.value.some((account) => account.account_id === normalizedAccountId)) return
      actions.updateCompose({ accountId: composeSendAccountOptions.value[0].account_id })
    },
    { immediate: true },
  )

  watch(
    () => ({
      isOpen: isComposeDialogOpen.value,
      ccText: props.composeForm?.ccText ?? '',
      bccText: props.composeForm?.bccText ?? '',
    }),
    ({ isOpen, ccText, bccText }) => {
      if (!isOpen) {
        isCcVisible.value = false
        isBccVisible.value = false
        isComposeCloseConfirmOpen.value = false
        return
      }
      if (ccText.trim()) isCcVisible.value = true
      if (bccText.trim()) isBccVisible.value = true
    },
    { immediate: true },
  )

  function handleToggleInspector(): void {
    isInspectorVisible.value = !isInspectorVisible.value
    actions.toggleInspector()
  }

  function toggleComposeEdgePanel(panelId: ComposeEdgePanelId): void {
    if (panelId === 'ai') {
      isAiComposePanelOpen.value = !isAiComposePanelOpen.value
      return
    }
    isContextComposePanelOpen.value = !isContextComposePanelOpen.value
  }

  function closeComposeEdgePanels(): void {
    isAiComposePanelOpen.value = false
    isContextComposePanelOpen.value = false
  }

  function showCcField(): void {
    isCcVisible.value = true
  }

  function showBccField(): void {
    isBccVisible.value = true
  }

  function hasComposeContent(): boolean {
    return composeFormHasTypedContent(props.composeForm)
  }

  function closeComposeNow(): void {
    isComposeCloseConfirmOpen.value = false
    closeComposeEdgePanels()
    actions.closeCompose()
  }

  function requestComposeClose(): void {
    if (hasComposeContent()) {
      isComposeCloseConfirmOpen.value = true
      return
    }
    closeComposeNow()
  }

  function handleComposeDialogOpenChange(open: boolean): void {
    if (!open) requestComposeClose()
  }

  function handleComposeEscape(): void {
    if (isAiComposePanelOpen.value || isContextComposePanelOpen.value) {
      closeComposeEdgePanels()
      return
    }
    requestComposeClose()
  }

  function handleComposeCloseConfirmOpenChange(open: boolean): void {
    isComposeCloseConfirmOpen.value = open
  }

  function handleDiscardComposeDraft(): void {
    closeComposeNow()
  }

  function handleSaveComposeDraftAndClose(): void {
    actions.saveCompose()
    closeComposeNow()
  }

  function handleComposeBodyHtmlChange(bodyHtml: string): void {
    actions.updateCompose({ body: composeBodyHtmlToPlainText(bodyHtml), bodyHtml, bodyFormat: 'html' })
  }

  function handleComposeDrop(files?: FileList | null): void {
    isComposeDropActive.value = false
    const selected = files ? Array.from(files) : []
    if (selected.length > 0) actions.attachComposeFiles(selected)
  }

  return {
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
    closeComposeEdgePanels,
    showCcField,
    showBccField,
    handleComposeDialogOpenChange,
    handleComposeEscape,
    requestComposeClose,
    handleComposeCloseConfirmOpenChange,
    handleDiscardComposeDraft,
    handleSaveComposeDraftAndClose,
    handleComposeBodyHtmlChange,
    handleComposeDrop,
  }
}
