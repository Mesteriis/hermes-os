import { useQueryClient } from '@tanstack/vue-query'
import { useTelegramMediaUploadMutation } from '../../platform/bootstrap/useTelegramMediaUploadWorkflow'
import type {
  TelegramConversationRuntimeActionRequest,
} from '../../shared/communications/types/telegramRuntimeActions'
import {
  useArchiveTelegramChatMutation,
  useAddTelegramChatToFolderMutation,
  useDownloadTelegramMediaMutation,
  useMarkReadTelegramChatMutation,
  useMarkUnreadTelegramChatMutation,
  useMuteTelegramChatMutation,
  usePinTelegramChatMutation,
  useReassignTelegramChatFoldersMutation,
  useRemoveTelegramChatFromFolderMutation,
  useSyncTelegramHistoryMutation,
  useUnarchiveTelegramChatMutation,
  useUnmuteTelegramChatMutation,
  useUnpinTelegramChatMutation,
} from '../../integrations/telegram/queries/useTelegramMutations'
import { useSyncTelegramChatMembersMutation } from '../../integrations/telegram/queries/useTelegramMembersQuery'
import { useJoinTelegramChatMutation, useLeaveTelegramChatMutation } from '../../integrations/telegram/queries/useTelegramParticipantLifecycleQuery'

export function useTelegramConversationRuntimeActions() {
  const queryClient = useQueryClient()
  const archive = useArchiveTelegramChatMutation()
  const addFolder = useAddTelegramChatToFolderMutation()
  const downloadMedia = useDownloadTelegramMediaMutation()
  const join = useJoinTelegramChatMutation()
  const leave = useLeaveTelegramChatMutation()
  const markRead = useMarkReadTelegramChatMutation()
  const markUnread = useMarkUnreadTelegramChatMutation()
  const mute = useMuteTelegramChatMutation()
  const pin = usePinTelegramChatMutation()
  const reassignFolders = useReassignTelegramChatFoldersMutation()
  const removeFolder = useRemoveTelegramChatFromFolderMutation()
  const syncMembers = useSyncTelegramChatMembersMutation()
  const syncHistory = useSyncTelegramHistoryMutation()
  const unarchive = useUnarchiveTelegramChatMutation()
  const unmute = useUnmuteTelegramChatMutation()
  const unpin = useUnpinTelegramChatMutation()
  const uploadMedia = useTelegramMediaUploadMutation()

  async function run(request: TelegramConversationRuntimeActionRequest): Promise<void> {
    const target = {
      telegramChatId: request.telegramChatId,
      accountId: request.accountId,
      providerChatId: request.providerChatId,
    }

    switch (request.action) {
      case 'archive':
        await archive.mutateAsync(target)
        break
      case 'download_media':
        if (!request.providerMessageId || request.tdlibFileId == null) {
          throw new Error('Telegram media download requires provider message and TDLib file ids.')
        }
        await downloadMedia.mutateAsync({
          account_id: request.accountId,
          provider_chat_id: request.providerChatId,
          provider_message_id: request.providerMessageId,
          tdlib_file_id: request.tdlibFileId,
          provider_attachment_id: request.providerAttachmentId,
          filename: request.filename,
          content_type: request.contentType,
        })
        break
      case 'folder_add':
        if (request.providerFolderId == null) throw new Error('Select a Telegram provider folder.')
        await addFolder.mutateAsync({ ...target, providerFolderId: request.providerFolderId })
        break
      case 'folder_remove':
        if (request.providerFolderId == null) throw new Error('Select a Telegram provider folder.')
        await removeFolder.mutateAsync({ ...target, providerFolderId: request.providerFolderId })
        break
      case 'folder_reassign':
        await reassignFolders.mutateAsync({ ...target, targetProviderFolderIds: request.providerFolderIds ?? [] })
        break
      case 'join':
        await join.mutateAsync(target)
        break
      case 'leave':
        await leave.mutateAsync(target)
        break
      case 'mark_read':
        await markRead.mutateAsync({
          ...target,
          lastReadInboxProviderMessageId: request.lastReadInboxProviderMessageId,
        })
        break
      case 'mark_unread':
        await markUnread.mutateAsync(target)
        break
      case 'mute':
        await mute.mutateAsync(target)
        break
      case 'pin':
        await pin.mutateAsync(target)
        break
      case 'sync_latest':
        await syncHistory.mutateAsync({
          account_id: request.accountId,
          provider_chat_id: request.providerChatId,
          mode: 'latest',
        })
        break
      case 'sync_members':
        await syncMembers.mutateAsync(request.telegramChatId)
        break
      case 'sync_older':
        if (request.historyFromMessageId == null || request.historyFromMessageId <= 0) {
          throw new Error('Telegram older-history sync requires the oldest loaded TDLib message id.')
        }
        await syncHistory.mutateAsync({
          account_id: request.accountId,
          provider_chat_id: request.providerChatId,
          mode: 'older',
          from_message_id: request.historyFromMessageId,
          limit: 100,
        })
        break
      case 'sync_full':
        await syncHistory.mutateAsync({
          account_id: request.accountId,
          provider_chat_id: request.providerChatId,
          mode: 'full',
          limit: 100,
        })
        break
      case 'unarchive':
        await unarchive.mutateAsync(target)
        break
      case 'unmute':
        await unmute.mutateAsync(target)
        break
      case 'unpin':
        await unpin.mutateAsync(target)
        break
      case 'upload_media':
        if (!request.file) throw new Error('Select a file before sending Telegram media.')
        await uploadMedia.mutateAsync({
          accountId: request.accountId,
          providerChatId: request.providerChatId,
          file: request.file,
          caption: request.caption,
        })
        break
    }

    await queryClient.invalidateQueries({ queryKey: ['communications', 'telegram'] })
  }

  return { run }
}
