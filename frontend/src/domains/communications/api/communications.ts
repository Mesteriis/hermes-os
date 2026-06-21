export {
  fetchCommunicationMessages,
  fetchCommunicationMessage,
  transitionMessageWorkflowState,
  fetchMessageStateCounts,
  trashMessage,
  restoreMessage,
  bulkMessageAction,
  analyzeMessage,
  runWorkflowAction,
  fetchMessageExplain,
  fetchMessageSmartCc,
  markMessageRead,
  deleteMessageFromProvider,
  toggleMessagePin,
  toggleMessageImportant,
  toggleMessageMute,
  snoozeMessage,
  addMessageLabel,
  exportMessage,
  fetchMessageAuth,
  fetchMessageSignature,
  detectMessageLanguage,
  translateMessage,
  createDraft,
  deleteDraft,
  fetchDrafts,
  searchEmails,
  fetchSubscriptions,
  fetchCommunicationBlockers,
  fetchPersonas,
  fetchRichTemplates,
  saveRichTemplate,
  deleteRichTemplate,
  renderRichTemplate,
  previewRichTemplateMailMerge,
  fetchTopSenders,
  fetchMailboxHealth
} from './messageApi'
export {
  fetchSavedSearches,
  createSavedSearch,
  updateSavedSearch,
  deleteSavedSearch
} from './savedSearchApi'
export {
  fetchCommunicationFolders,
  createCommunicationFolder,
  updateCommunicationFolder,
  deleteCommunicationFolder,
  fetchFolderMessages,
  copyMessageToFolder,
  moveMessageToFolder
} from './folderApi'
export {
  fetchMailSyncStatus,
  fetchMailSyncSettings,
  updateMailSyncSettings,
  runMailSyncNow,
  runMailFullResync
} from '../../../shared/mailSync/syncApi'
export { sendEmail, redirectMessage } from './sendApi'
export {
  fetchOutboxItems,
  undoOutboxItem
} from './outboxApi'
export {
  fetchThreads,
  fetchThreadMessages,
  translateThread
} from './threadApi'
export {
  searchAttachments,
  inspectAttachmentArchive,
  previewAttachment,
  translateAttachment
} from './attachmentApi'
export {
  createMailCertificate,
  fetchExpiringMailCertificates,
  fetchMailCertificates
} from './certificateApi'
export {
  fetchMessageAiState,
  updateMessageAiState
} from './aiState'
export {
  generateAiReply,
  generateAiReplyVariants,
  extractMessageTasks,
  extractMessageNotes
} from './messageApi'
export type { CommunicationSearchResponse } from '../types/communications'
