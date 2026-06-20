import { useMutation, useQueryClient } from '@tanstack/vue-query'
import {
  analyzeMessage,
  addMessageLabel,
  bulkMessageAction,
  exportMessage,
  deleteMessageFromProvider,
  markMessageRead,
  extractMessageNotes,
  extractMessageTasks,
  fetchMessageAuth,
  fetchMessageExplain,
  fetchMessageSignature,
  fetchMessageSmartCc,
  detectMessageLanguage,
  generateAiReply,
  generateAiReplyVariants,
  runMailFullResync,
  runMailSyncNow,
  toggleMessageImportant,
  toggleMessageMute,
  toggleMessagePin,
  snoozeMessage,
  translateThread,
  translateMessage
} from '../api/communications'
import type {
  BulkMessageActionResponse,
  ExtractNotesResponse,
  ExtractTasksResponse,
  MailSyncRunResponse,
  MessageAnalyzeResponse,
  AiReplyResponse,
  AiReplyVariantsResponse,
  LanguageDetection,
  MessageAuthCheckResponse,
  MessageExplainResponse,
  MessageExportResponse,
  MessageExportFormat,
  MessageImportantToggleResponse,
  MessagePinToggleResponse,
  LocalMessageStateResponse,
  SignatureDetection,
  SmartCcResponse,
  TranslationResponse
} from '../types/communications'
import type { ThreadTranslationResponse } from '../types/multilingual'

function invalidateMessageViews(queryClient: ReturnType<typeof useQueryClient>, messageId: string) {
  queryClient.invalidateQueries({ queryKey: ['communications-message', messageId] })
  queryClient.invalidateQueries({ queryKey: ['communications-mail-list'] })
}

function invalidateSyncViews(queryClient: ReturnType<typeof useQueryClient>) {
  queryClient.invalidateQueries({ queryKey: ['communications-mail-list'] })
  queryClient.invalidateQueries({ queryKey: ['communications-state-counts'] })
  queryClient.invalidateQueries({ queryKey: ['integrations', 'mail', 'sync-statuses'] })
  queryClient.invalidateQueries({ queryKey: ['integrations', 'mail', 'mailbox-health'] })
}

export function useToggleMessagePinMutation() {
  const queryClient = useQueryClient()
  return useMutation<MessagePinToggleResponse, Error, string>({
    mutationFn: async (messageId) => toggleMessagePin(messageId),
    onSuccess: (_result, messageId) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useToggleMessageImportantMutation() {
  const queryClient = useQueryClient()
  return useMutation<MessageImportantToggleResponse, Error, string>({
    mutationFn: async (messageId) => toggleMessageImportant(messageId),
    onSuccess: (_result, messageId) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useToggleMessageMuteMutation() {
  const queryClient = useQueryClient()
  return useMutation<MessagePinToggleResponse, Error, string>({
    mutationFn: async (messageId) => toggleMessageMute(messageId),
    onSuccess: (_result, messageId) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useExportMessageMutation() {
  return useMutation<MessageExportResponse, Error, { messageId: string; format: MessageExportFormat }>({
    mutationFn: async ({ messageId, format }) => exportMessage(messageId, format)
  })
}

export function useMarkMessageReadMutation() {
  const queryClient = useQueryClient()
  return useMutation<Record<string, unknown>, Error, string>({
    mutationFn: async (messageId) => markMessageRead(messageId),
    onSuccess: (_result, messageId) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useDeleteMessageFromProviderMutation() {
  const queryClient = useQueryClient()
  return useMutation<LocalMessageStateResponse, Error, string>({
    mutationFn: async (messageId) => deleteMessageFromProvider(messageId),
    onSuccess: (_result, messageId) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useMarkMessageUnreadMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    BulkMessageActionResponse,
    Error,
    string
  >({
    mutationFn: async (messageId) => bulkMessageAction({ action: 'mark_unread', message_ids: [messageId] }),
    onSuccess: (_result, messageId) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useAddMessageLabelMutation() {
  const queryClient = useQueryClient()
  return useMutation<Record<string, unknown>, Error, { messageId: string; label: string }>({
    mutationFn: async ({ messageId, label }) => addMessageLabel(messageId, label),
    onSuccess: (_result, { messageId }) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useRemoveMessageLabelMutation() {
  const queryClient = useQueryClient()
  return useMutation<BulkMessageActionResponse, Error, { messageId: string; label: string }>({
    mutationFn: async ({ messageId, label }) =>
      bulkMessageAction({ action: 'remove_label', message_ids: [messageId], label }),
    onSuccess: (_result, { messageId }) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useSnoozeMessageMutation() {
  const queryClient = useQueryClient()
  return useMutation<Record<string, unknown>, Error, { messageId: string; until: string }>({
    mutationFn: async ({ messageId, until }) => snoozeMessage(messageId, until),
    onSuccess: (_result, { messageId }) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useAnalyzeMessageMutation() {
  const queryClient = useQueryClient()
  return useMutation<MessageAnalyzeResponse, Error, string>({
    mutationFn: async (messageId) => analyzeMessage(messageId),
    onSuccess: (_result, messageId) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useGenerateAiReplyMutation() {
  return useMutation<AiReplyResponse, Error, { messageId: string; tone: string; language: string }>({
    mutationFn: async ({ messageId, tone, language }) => generateAiReply(messageId, { tone, language })
  })
}

export function useGenerateAiReplyVariantsMutation() {
  return useMutation<
    AiReplyVariantsResponse,
    Error,
    { messageId: string; languages: string[]; tones: string[] }
  >({
    mutationFn: async ({ messageId, languages, tones }) =>
      generateAiReplyVariants(messageId, { languages, tones })
  })
}

export function useExplainMessageMutation() {
  return useMutation<MessageExplainResponse, Error, string>({
    mutationFn: async (messageId) => fetchMessageExplain(messageId)
  })
}

export function useDetectMessageLanguageMutation() {
  return useMutation<LanguageDetection, Error, string>({
    mutationFn: async (messageId) => detectMessageLanguage(messageId)
  })
}

export function useReviewMessageSecurityMutation() {
  return useMutation<
    { auth: MessageAuthCheckResponse; signature: SignatureDetection },
    Error,
    string
  >({
    mutationFn: async (messageId) => {
      const [auth, signature] = await Promise.all([
        fetchMessageAuth(messageId),
        fetchMessageSignature(messageId)
      ])
      return { auth, signature }
    }
  })
}

export function useReviewMessageRecipientsMutation() {
  return useMutation<SmartCcResponse, Error, string>({
    mutationFn: async (messageId) => fetchMessageSmartCc(messageId)
  })
}

export function useTranslateMessageMutation() {
  return useMutation<TranslationResponse, Error, { messageId: string; targetLanguage: string }>({
    mutationFn: async ({ messageId, targetLanguage }) => translateMessage(messageId, targetLanguage)
  })
}

export function useTranslateThreadMutation() {
  return useMutation<
    ThreadTranslationResponse,
    Error,
    { accountId: string; subject: string; targetLanguage: string; limit?: number }
  >({
    mutationFn: async ({ accountId, subject, targetLanguage, limit }) =>
      translateThread(accountId, subject, targetLanguage, limit)
  })
}

export function useExtractMessageTasksMutation() {
  return useMutation<ExtractTasksResponse, Error, string>({
    mutationFn: async (messageId) => extractMessageTasks(messageId)
  })
}

export function useExtractMessageNotesMutation() {
  return useMutation<ExtractNotesResponse, Error, string>({
    mutationFn: async (messageId) => extractMessageNotes(messageId)
  })
}

export function useRunMailSyncNowMutation() {
  const queryClient = useQueryClient()
  return useMutation<MailSyncRunResponse, Error, string>({
    mutationFn: async (accountId) => runMailSyncNow(accountId),
    onSuccess: () => invalidateSyncViews(queryClient)
  })
}

export function useRunMailFullResyncMutation() {
  const queryClient = useQueryClient()
  return useMutation<MailSyncRunResponse, Error, string>({
    mutationFn: async (accountId) => runMailFullResync(accountId),
    onSuccess: () => invalidateSyncViews(queryClient)
  })
}
