import { useMutation, useQueryClient } from '@tanstack/vue-query'
import {
  addTelegramChatToFolder,
  archiveTelegramChat,
  downloadTelegramMedia,
  logoutTelegramAccount,
  markTelegramChatRead,
  markTelegramChatUnread,
  muteTelegramChat,
  pinTelegramChat,
  reassignTelegramChatFolders,
  removeTelegramAccount,
  removeTelegramChatFromFolder,
  setupTelegramAccount,
  syncTelegramChats,
  syncTelegramHistory,
  unarchiveTelegramChat,
  unmuteTelegramChat,
  unpinTelegramChat,
} from '../api/telegram'
import type {
  TelegramChatSyncRequest,
  TelegramHistorySyncRequest,
  TelegramMediaDownloadRequest,
} from '../types/telegram'
import { telegramQueryKeys } from './telegramQueryKeys'

export function useSetupTelegramAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: setupTelegramAccount,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.accounts })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.capabilities })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
    },
  })
}

export function useLogoutTelegramAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (accountId: string) => logoutTelegramAccount(accountId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.accounts })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.capabilities })
    },
  })
}

export function useRemoveTelegramAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (accountId: string) => removeTelegramAccount(accountId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.accounts })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.capabilities })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.folders })
    },
  })
}

export function useSyncTelegramChatsMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: TelegramChatSyncRequest) => syncTelegramChats(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.folders })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
    },
  })
}

export function useSyncTelegramHistoryMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: TelegramHistorySyncRequest) => syncTelegramHistory(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    },
  })
}

export function useDownloadTelegramMediaMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: TelegramMediaDownloadRequest) => downloadTelegramMedia(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
    },
  })
}

function useTelegramChatLifecycleMutation(
  mutationFn: (args: {
    telegramChatId: string
    accountId: string
    providerChatId: string
  }) => Promise<unknown>,
  invalidateFolders = false,
  invalidateDetail = false
) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      if (invalidateFolders) {
        queryClient.invalidateQueries({ queryKey: telegramQueryKeys.folders })
      }
      if (invalidateDetail) {
        queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
      }
    },
  })
}

export function usePinTelegramChatMutation() {
  return useTelegramChatLifecycleMutation(({ telegramChatId, accountId, providerChatId }) =>
    pinTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }))
}

export function useUnpinTelegramChatMutation() {
  return useTelegramChatLifecycleMutation(({ telegramChatId, accountId, providerChatId }) =>
    unpinTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }))
}

export function useArchiveTelegramChatMutation() {
  return useTelegramChatLifecycleMutation(({ telegramChatId, accountId, providerChatId }) =>
    archiveTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }))
}

export function useUnarchiveTelegramChatMutation() {
  return useTelegramChatLifecycleMutation(({ telegramChatId, accountId, providerChatId }) =>
    unarchiveTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }))
}

export function useMuteTelegramChatMutation() {
  return useTelegramChatLifecycleMutation(({ telegramChatId, accountId, providerChatId }) =>
    muteTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }))
}

export function useUnmuteTelegramChatMutation() {
  return useTelegramChatLifecycleMutation(({ telegramChatId, accountId, providerChatId }) =>
    unmuteTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }))
}

export function useAddTelegramChatToFolderMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId, providerFolderId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
      providerFolderId: number
    }) => addTelegramChatToFolder(telegramChatId, providerFolderId, {
      account_id: accountId,
      provider_chat_id: providerChatId,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.folders })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
    },
  })
}

export function useRemoveTelegramChatFromFolderMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId, providerFolderId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
      providerFolderId: number
    }) => removeTelegramChatFromFolder(telegramChatId, providerFolderId, {
      account_id: accountId,
      provider_chat_id: providerChatId,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.folders })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
    },
  })
}

export function useReassignTelegramChatFoldersMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId, targetProviderFolderIds }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
      targetProviderFolderIds: number[]
    }) => reassignTelegramChatFolders(telegramChatId, {
      account_id: accountId,
      provider_chat_id: providerChatId,
      target_provider_folder_ids: targetProviderFolderIds,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.folders })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
    },
  })
}

export function useMarkReadTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId, lastReadInboxProviderMessageId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
      lastReadInboxProviderMessageId?: string
    }) => markTelegramChatRead(telegramChatId, {
      account_id: accountId,
      provider_chat_id: providerChatId,
      ...(lastReadInboxProviderMessageId
        ? {
            last_read_inbox_provider_message_id: lastReadInboxProviderMessageId,
          }
        : {}),
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
    },
  })
}

export function useMarkUnreadTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
    }) => markTelegramChatUnread(telegramChatId, {
      account_id: accountId,
      provider_chat_id: providerChatId,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
    },
  })
}
