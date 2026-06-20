import type {
  TelegramCapabilitiesResponse,
  TelegramChat,
  TelegramMediaItem,
  TelegramMessage,
  TelegramProviderKind,
} from '../types/telegram'
import { telegramChatFolderIds } from '../folderMembership'
import { telegramLatestReadableProviderMessageId } from '../stores/telegramReadProgress'

type TelegramChatActionMutation = {
  mutateAsync: (args: {
    telegramChatId: string
    accountId: string
    providerChatId: string
    lastReadInboxProviderMessageId?: string
  }) => Promise<unknown>
}

type TelegramForwardMessageMutation = {
  mutateAsync: (args: {
    message_id: string
    account_id: string
    provider_chat_id: string
    from_provider_chat_id: string
    from_provider_message_id: string
  }) => Promise<{ provider_chat_id: string; status: string }>
}

type TelegramMessageReadMutation = {
  mutateAsync: (args: {
    message_id: string
    account_id: string
    provider_chat_id: string
  }) => Promise<{ status: string }>
}

type TelegramChatFolderMutation = {
  mutateAsync: (args: {
    telegramChatId: string
    accountId: string
    providerChatId: string
    providerFolderId: number
  }) => Promise<{ status: string }>
}

type TelegramChatFolderReassignMutation = {
  mutateAsync: (args: {
    telegramChatId: string
    accountId: string
    providerChatId: string
    targetProviderFolderIds: number[]
  }) => Promise<{ status: string }>
}

type TelegramChatToggleActionParams = {
  chat: TelegramChat
  isActive: boolean
  activateMutation: TelegramChatActionMutation
  deactivateMutation: TelegramChatActionMutation
  activateMessage: string
  deactivateMessage: string
  setSubmitting: (value: boolean) => void
  setActionMessage: (value: string) => void
  setError: (value: string) => void
  activateVariables?: {
    lastReadInboxProviderMessageId?: string
  }
}

type TelegramForwardMessageActionParams = {
  chat: TelegramChat
  message: TelegramMessage
  mutation: TelegramForwardMessageMutation
  sourceChatUnavailableMessage: string
  setSubmitting: (value: boolean) => void
  setActionMessage: (value: string) => void
  setError: (value: string) => void
  setSelectedChatId: (value: string) => void
}

type TelegramMessageReadActionParams = {
  chat: TelegramChat
  message: TelegramMessage
  mutation: TelegramMessageReadMutation
  setSubmitting: (value: boolean) => void
  setActionMessage: (value: string) => void
  setError: (value: string) => void
}

type TelegramSearchNavigationCallbacks = {
  setError: (value: string) => void
  selectChat: (chat: TelegramChat) => void
  focusMessage: (message: TelegramMessage) => void
  clearFocusedMessage: () => void
  setActiveThreadTab: (tab: 'messages') => void
  setSearchQuery: (value: string) => void
}

type TelegramChatFolderActionParams = {
  chat: TelegramChat
  providerFolderId: number
  mutation: TelegramChatFolderMutation
  setSubmitting: (value: boolean) => void
  setActionMessage: (value: string) => void
  setError: (value: string) => void
}

type TelegramChatFolderReassignActionParams = {
  chat: TelegramChat
  targetProviderFolderIds: number[]
  mutation: TelegramChatFolderReassignMutation
  setSubmitting: (value: boolean) => void
  setActionMessage: (value: string) => void
  setError: (value: string) => void
}

export function isTelegramChatPinned(chat: TelegramChat): boolean {
  return Boolean(chat.metadata.is_pinned ?? chat.metadata.pinned)
}

export function isTelegramChatArchived(chat: TelegramChat): boolean {
  return Boolean(chat.metadata.is_archived)
}

export function isTelegramChatMuted(chat: TelegramChat): boolean {
  return Boolean(chat.metadata.is_muted ?? chat.metadata.muted)
}

export function telegramChatUnreadCountValue(chat: TelegramChat): number {
  const value = chat.metadata.unread_count
  return typeof value === 'number' ? value : 0
}

export async function runTelegramChatToggleAction({
  chat,
  isActive,
  activateMutation,
  deactivateMutation,
  activateMessage,
  deactivateMessage,
  setSubmitting,
  setActionMessage,
  setError,
  activateVariables,
}: TelegramChatToggleActionParams): Promise<void> {
  setSubmitting(true)
  setActionMessage('')
  setError('')
  try {
    await (isActive ? deactivateMutation : activateMutation).mutateAsync({
      telegramChatId: chat.telegram_chat_id,
      accountId: chat.account_id,
      providerChatId: chat.provider_chat_id,
      ...(!isActive ? activateVariables : undefined),
    })
    setActionMessage(isActive ? deactivateMessage : activateMessage)
  } catch (error) {
    setError(error instanceof Error ? error.message : String(error))
  } finally {
    setSubmitting(false)
  }
}

export async function runTelegramChatReadToggleAction(
  chat: TelegramChat,
  messages: TelegramMessage[],
  markReadMutation: TelegramChatActionMutation,
  markUnreadMutation: TelegramChatActionMutation,
  setSubmitting: (value: boolean) => void,
  setActionMessage: (value: string) => void,
  setError: (value: string) => void,
): Promise<void> {
  const lastReadInboxProviderMessageId = telegramLatestReadableProviderMessageId(chat, messages)
  await runTelegramChatToggleAction({
    chat,
    isActive: telegramChatUnreadCountValue(chat) === 0,
    activateMutation: markReadMutation,
    deactivateMutation: markUnreadMutation,
    activateMessage: 'Chat marked read locally',
    deactivateMessage: 'Chat marked unread locally',
    setSubmitting,
    setActionMessage,
    setError,
    activateVariables: lastReadInboxProviderMessageId
      ? { lastReadInboxProviderMessageId }
      : undefined,
  })
}

export async function runTelegramForwardMessageAction({
  chat,
  message,
  mutation,
  sourceChatUnavailableMessage,
  setSubmitting,
  setActionMessage,
  setError,
  setSelectedChatId,
}: TelegramForwardMessageActionParams): Promise<void> {
  const sourceProviderChatId = message.provider_chat_id ?? chat.provider_chat_id
  if (!sourceProviderChatId) {
    setError(sourceChatUnavailableMessage)
    return
  }
  setSubmitting(true)
  setActionMessage('')
  setError('')
  try {
    const result = await mutation.mutateAsync({
      message_id: message.message_id,
      account_id: chat.account_id,
      provider_chat_id: chat.provider_chat_id,
      from_provider_chat_id: sourceProviderChatId,
      from_provider_message_id: message.provider_message_id,
    })
    setSelectedChatId(result.provider_chat_id)
    setActionMessage(`Telegram forward ${result.status}`)
  } catch (error) {
    setError(error instanceof Error ? error.message : String(error))
  } finally {
    setSubmitting(false)
  }
}

export async function runTelegramMessageReadAction({
  chat,
  message,
  mutation,
  setSubmitting,
  setActionMessage,
  setError,
}: TelegramMessageReadActionParams): Promise<void> {
  setSubmitting(true)
  setActionMessage('')
  setError('')
  try {
    const result = await mutation.mutateAsync({
      message_id: message.message_id,
      account_id: message.account_id,
      provider_chat_id: message.provider_chat_id ?? chat.provider_chat_id,
    })
    setActionMessage(`Message mark-read ${result.status}`)
  } catch (error) {
    setError(error instanceof Error ? error.message : String(error))
  } finally {
    setSubmitting(false)
  }
}

export function telegramCapabilityEnabled(
  capabilities: TelegramCapabilitiesResponse | null | undefined,
  operation: string
): boolean {
  const status = capabilities?.capabilities.find((item) => item.operation === operation)?.status
  return status === 'available' || status === 'degraded'
}

export function telegramCapabilityReason(
  capabilities: TelegramCapabilitiesResponse | null | undefined,
  operation: string,
  fallback: string
): string {
  return capabilities?.capabilities.find((item) => item.operation === operation)?.reason ?? fallback
}

export async function runTelegramAddChatToFolderAction({
  chat,
  providerFolderId,
  mutation,
  setSubmitting,
  setActionMessage,
  setError,
}: TelegramChatFolderActionParams): Promise<void> {
  setSubmitting(true)
  setActionMessage('')
  setError('')
  try {
    const result = await mutation.mutateAsync({
      telegramChatId: chat.telegram_chat_id,
      accountId: chat.account_id,
      providerChatId: chat.provider_chat_id,
      providerFolderId,
    })
    setActionMessage(`Telegram folder command ${result.status}`)
  } catch (error) {
    setError(error instanceof Error ? error.message : String(error))
  } finally {
    setSubmitting(false)
  }
}

export async function runTelegramReassignChatFoldersAction({
  chat,
  targetProviderFolderIds,
  mutation,
  setSubmitting,
  setActionMessage,
  setError,
}: TelegramChatFolderReassignActionParams): Promise<void> {
  setSubmitting(true)
  setActionMessage('')
  setError('')
  try {
    const result = await mutation.mutateAsync({
      telegramChatId: chat.telegram_chat_id,
      accountId: chat.account_id,
      providerChatId: chat.provider_chat_id,
      targetProviderFolderIds,
    })
    setActionMessage(`Telegram folder reassignment ${result.status}`)
  } catch (error) {
    setError(error instanceof Error ? error.message : String(error))
  } finally {
    setSubmitting(false)
  }
}

export function telegramChatHasFolder(chat: TelegramChat, providerFolderId: number): boolean {
  const folderIds = telegramChatFolderIds(chat)
  if (folderIds.includes(providerFolderId)) {
    return true
  }
  return chat.metadata.provider_folder_id === providerFolderId
}

export function telegramChatNeedsFolderReassign(chat: TelegramChat, providerFolderId: number): boolean {
  const folderIds = telegramChatFolderIds(chat)
  return folderIds.length !== 1 || folderIds[0] !== providerFolderId
}

export function hasProjectedTelegramMessagesForChat(
  messages: TelegramMessage[],
  chat: TelegramChat
): boolean {
  return messages.some(
    (message) =>
      message.account_id === chat.account_id &&
      message.provider_chat_id === chat.provider_chat_id
  )
}

export function findTelegramChatForMessage(
  chats: TelegramChat[],
  message: TelegramMessage
): TelegramChat | null {
  return chats.find(
    (chat) =>
      chat.account_id === message.account_id &&
      chat.provider_chat_id === message.provider_chat_id
  ) ?? null
}

export function buildFocusedTelegramMessageFromMediaSearch(params: {
  item: TelegramMediaItem
  chat: TelegramChat
  providerKind: TelegramProviderKind
}): TelegramMessage {
  return {
    message_id: params.item.message_id,
    raw_record_id: `telegram-media-search:${params.item.message_id}`,
    account_id: params.chat.account_id,
    provider_message_id: params.item.provider_message_id,
    provider_chat_id: params.item.provider_chat_id,
    chat_title: params.chat.title,
    sender: params.chat.title,
    sender_display_name: params.chat.title,
    text: params.item.file_name,
    occurred_at: params.item.occurred_at,
    projected_at: params.item.occurred_at ?? new Date().toISOString(),
    channel_kind: params.providerKind,
    delivery_state: 'received',
    metadata: {},
  }
}

export function openTelegramSearchMessageInThread(
  chats: TelegramChat[],
  message: TelegramMessage,
  callbacks: TelegramSearchNavigationCallbacks
): void {
  const nextChat = findTelegramChatForMessage(chats, message)
  if (!nextChat) {
    callbacks.setError('Telegram search result chat is not loaded in the current projection.')
    return
  }
  callbacks.selectChat(nextChat)
  callbacks.focusMessage(message)
  callbacks.setSearchQuery('')
}

export function openTelegramSearchChatInThread(
  chat: TelegramChat,
  callbacks: TelegramSearchNavigationCallbacks
): void {
  callbacks.clearFocusedMessage()
  callbacks.selectChat(chat)
  callbacks.setActiveThreadTab('messages')
  callbacks.setSearchQuery('')
}

export function openTelegramSearchMediaInThread(params: {
  item: TelegramMediaItem
  currentChat: TelegramChat | null
  providerKind: TelegramProviderKind
  callbacks: TelegramSearchNavigationCallbacks
}): void {
  if (!params.currentChat || params.currentChat.provider_chat_id !== params.item.provider_chat_id) {
    params.callbacks.setError('Telegram media result chat is not loaded in the current projection.')
    return
  }
  params.callbacks.selectChat(params.currentChat)
  params.callbacks.focusMessage(
    buildFocusedTelegramMessageFromMediaSearch({
      item: params.item,
      chat: params.currentChat,
      providerKind: params.providerKind,
    })
  )
  params.callbacks.setActiveThreadTab('messages')
  params.callbacks.setSearchQuery('')
}

export function formatTelegramDateTime(date: string | null): string {
  if (!date) return ''
  try {
    return new Date(date).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch {
    return ''
  }
}
