import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
export { telegramRuntimeCommandTarget } from './telegramRuntimeStatus'
import type {
  TelegramChat,
  TelegramMessage,
  TelegramChatFilter,
  TelegramChatFilterCount,
  TelegramChatGroupFilter,
  TelegramRailTab,
  TelegramThreadTab,
  TelegramAttachmentHint,
  TelegramMediaItem,
} from '../types/telegram'

// --- Metadata helpers ---

function metaNumber(metadata: Record<string, unknown>, key: string): number {
  const v = metadata[key]
  return typeof v === 'number' ? v : 0
}

function metaBoolean(metadata: Record<string, unknown>, key: string): boolean {
  return Boolean(metadata[key])
}

function metaString(metadata: Record<string, unknown>, key: string): string {
  const v = metadata[key]
  return typeof v === 'string' ? v : ''
}

function metaStringArray(metadata: Record<string, unknown>, key: string): string[] {
  const value = metadata[key]
  if (!Array.isArray(value)) return []
  return value.filter((item): item is string => typeof item === 'string' && item.trim().length > 0)
}

// --- Helper functions (ported from Svelte services/telegram/messages.ts) ---

export function telegramMessagesChronological(messages: TelegramMessage[]): TelegramMessage[] {
  return messages.slice().sort((left, right) => {
    const la = left.occurred_at ?? left.projected_at ?? ''
    const ra = right.occurred_at ?? right.projected_at ?? ''
    return la.localeCompare(ra)
  })
}

export function telegramChatUnreadCount(chat: TelegramChat): number {
  return metaNumber(chat.metadata, 'unread_count')
}

export function telegramChatMentionCountValue(chat: TelegramChat): number {
  return metaNumber(chat.metadata, 'mention_count')
}

export function telegramChatLastReadInboxProviderMessageId(chat: TelegramChat): string {
  return metaString(chat.metadata, 'last_read_inbox_provider_message_id')
}

export function telegramChatIsPinned(chat: TelegramChat): boolean {
  return metaBoolean(chat.metadata, 'is_pinned') || metaBoolean(chat.metadata, 'pinned')
}

export function telegramChatIsProject(chat: TelegramChat): boolean {
  return metaBoolean(chat.metadata, 'is_project')
}

export function telegramChatIsBot(chat: TelegramChat): boolean {
  return chat.chat_kind === 'bot'
}

export function telegramChatIsSavedMessages(chat: TelegramChat): boolean {
  return metaBoolean(chat.metadata, 'is_saved_messages')
}

export function telegramChatTypingLabel(chat: TelegramChat, nowMs = Date.now()): string {
  const activeTyping = chat.metadata.active_typing
  if (!activeTyping || typeof activeTyping !== 'object') return ''
  const record = activeTyping as Record<string, unknown>
  if (record.is_active !== true) return ''
  const expiresAt = typeof record.expires_at === 'string' ? Date.parse(record.expires_at) : NaN
  if (Number.isFinite(expiresAt) && expiresAt <= nowMs) return ''
  const senderId = typeof record.sender_id === 'string' ? record.sender_id : ''
  return senderId ? `${senderId} typing...` : 'typing...'
}

export function telegramChatPreview(chat: TelegramChat, messages: TelegramMessage[]): string {
  const typing = telegramChatTypingLabel(chat)
  if (typing) return typing
  const relevant = messages
    .filter((m) => m.provider_chat_id === chat.provider_chat_id)
    .sort((left, right) => {
      const la = left.occurred_at ?? left.projected_at ?? ''
      const ra = right.occurred_at ?? right.projected_at ?? ''
      return ra.localeCompare(la)
    })
  const latest = relevant[0]
  if (!latest) return ''
  return latest.text?.slice(0, 100) ?? ''
}

export function telegramChatFilterCounts(
  chats: TelegramChat[],
  messages: TelegramMessage[]
): TelegramChatFilterCount[] {
  const all = chats.length
  const unread = chats.filter((c) => telegramChatUnreadCount(c) > 0).length
  const mentions = chats.filter((c) => telegramChatMentionCountValue(c) > 0).length
  const pinned = chats.filter((c) => telegramChatIsPinned(c)).length
  const projects = chats.filter((c) => telegramChatIsProject(c)).length
  const bots = chats.filter((c) => telegramChatIsBot(c)).length
  const archived = chats.filter((c) => metaBoolean(c.metadata, 'is_archived')).length
  return [
    { filter: 'all', count: all },
    { filter: 'unread', count: unread },
    { filter: 'mentions', count: mentions },
    { filter: 'pinned', count: pinned },
    { filter: 'projects', count: projects },
    { filter: 'bots', count: bots },
    { filter: 'archived', count: archived }
  ]
}

export function telegramFilterTabs(t: (key: string) => string): Array<{ id: TelegramChatFilter; label: string }> {
  return [
    { id: 'all', label: t('All Chats') },
    { id: 'unread', label: t('Unread') },
    { id: 'mentions', label: t('Mentions') },
    { id: 'pinned', label: t('Pinned') },
    { id: 'projects', label: t('Projects') },
    { id: 'bots', label: t('Bots') },
    { id: 'archived', label: t('Archived') }
  ]
}

export function telegramChatGroupFilters(chats: TelegramChat[]): TelegramChatGroupFilter[] {
  const groups: TelegramChatGroupFilter[] = [
    { id: 'local:all', label: 'All', icon: 'tabler:message', source: 'local', count: chats.length }
  ]
  const folders = new Map<string, { name: string; count: number }>()
  for (const chat of chats) {
    const folderNames = metaStringArray(chat.metadata, 'folder_labels')
    const labels = folderNames.length ? folderNames : (() => {
      const folderName = metaString(chat.metadata, 'folder_name')
      return folderName ? [folderName] : []
    })()
    for (const folderName of labels) {
      const existing = folders.get(folderName)
      if (existing) {
        existing.count++
      } else {
        folders.set(folderName, { name: folderName, count: 1 })
      }
    }
  }
  for (const [, folder] of folders) {
    groups.push({
      id: `folder:${folder.name}`,
      label: folder.name,
      icon: 'tabler:folder',
      source: 'telegram',
      count: folder.count
    })
  }
  return groups
}

export function filterTelegramChatsByGroup(
  chats: TelegramChat[],
  groupFilter: string
): TelegramChat[] {
  if (groupFilter === 'local:all') return chats
  if (groupFilter.startsWith('folder:')) {
    const folderName = groupFilter.slice(7)
    return chats.filter((chat) => {
      const labels = metaStringArray(chat.metadata, 'folder_labels')
      if (labels.length) return labels.includes(folderName)
      return metaString(chat.metadata, 'folder_name') === folderName
    })
  }
  return chats
}

export function filterTelegramChats(
  chats: TelegramChat[],
  messages: TelegramMessage[],
  searchQuery: string,
  filter: TelegramChatFilter
): TelegramChat[] {
  return chats.filter((chat) => {
    if (searchQuery.trim()) {
      const q = searchQuery.trim().toLowerCase()
      if (!chat.title.toLowerCase().includes(q)) return false
    }
    switch (filter) {
      case 'all':
        return true
      case 'unread':
        return telegramChatUnreadCount(chat) > 0
      case 'mentions':
        return telegramChatMentionCountValue(chat) > 0
      case 'pinned':
        return telegramChatIsPinned(chat)
      case 'projects':
        return telegramChatIsProject(chat)
      case 'bots':
        return telegramChatIsBot(chat)
      case 'archived':
        return metaBoolean(chat.metadata, 'is_archived')
      default:
        return true
    }
  })
}

export function telegramMessageAttachmentHints(
  message: TelegramMessage
): TelegramAttachmentHint[] {
  const rawAttachments = message.metadata?.attachments ?? message.metadata?.files
  if (!Array.isArray(rawAttachments) || !rawAttachments.length) return []
  return rawAttachments.map((att: Record<string, unknown>, idx: number) => ({
    id: (att.id as string) ?? `${message.message_id}:${idx}`,
    kind: ((att.attachment_type as string) ?? 'file') as TelegramAttachmentHint['kind'],
    fileName: (att.filename as string) ?? `attachment-${idx}`,
    mimeType: (att.content_type as string) ?? null,
    sizeBytes: (att.size as number) ?? null,
    tdlibFileId: (att.tdlib_file_id as number) ?? (att.metadata as Record<string, unknown>)?.tdlib_file_id as number ?? null,
    providerAttachmentId: (att.attachment_id as string) ?? '',
    downloadState: ((att.download_state as string) ?? 'unknown') as TelegramAttachmentHint['downloadState'],
    localPath: (att.local_path as string) ?? null,
    expectedSizeBytes: (att.expected_size_bytes as number) ?? null,
    downloadedSizeBytes: (att.downloaded_size_bytes as number) ?? null,
    isDownloadingActive: (att.is_downloading_active as boolean) ?? null,
    isDownloadingCompleted: (att.is_downloading_completed as boolean) ?? null,
    lastError: (att.last_error as string) ?? null,
    messageId: message.message_id,
    providerMessageId: message.provider_message_id,
  }))
}

export function telegramAttachmentHintFromMediaItem(
  item: TelegramMediaItem
): TelegramAttachmentHint {
  return {
    id: `${item.message_id}:${item.file_name}`,
    kind: (item.kind as TelegramAttachmentHint['kind']) ?? 'file',
    fileName: item.file_name,
    mimeType: item.mime_type,
    sizeBytes: item.size_bytes,
    tdlibFileId: item.tdlib_file_id,
    providerAttachmentId: item.provider_attachment_id ?? '',
    downloadState: (item.download_state as TelegramAttachmentHint['downloadState']) ?? 'unknown',
    localPath: item.local_path,
    expectedSizeBytes: item.expected_size_bytes ?? null,
    downloadedSizeBytes: item.downloaded_size_bytes ?? null,
    isDownloadingActive: item.is_downloading_active ?? null,
    isDownloadingCompleted: item.is_downloading_completed ?? null,
    lastError: item.last_error ?? null,
    messageId: item.message_id,
    providerMessageId: item.provider_message_id,
  }
}

export function telegramAttachmentHintsForMessages(
  messages: TelegramMessage[]
): ReturnType<typeof telegramMessageAttachmentHints> {
  const hints: ReturnType<typeof telegramMessageAttachmentHints> = []
  const seen = new Set<string>()
  for (const message of messages) {
    for (const hint of telegramMessageAttachmentHints(message)) {
      if (!seen.has(hint.fileName)) {
        seen.add(hint.fileName)
        hints.push(hint)
      }
    }
  }
  return hints
}

export function telegramMediaAlbumGroupsForMessages(messages: TelegramMessage[]): Array<{
  albumKey: string
  albumId: string
  messages: TelegramMessage[]
  attachmentCount: number
  occurredAt: string | null
}> {
  const groups = new Map<string, {
    albumKey: string
    albumId: string
    messages: TelegramMessage[]
    attachmentCount: number
    occurredAt: string | null
  }>()

  for (const message of messages) {
    const albumKey = metaString(message.metadata, 'media_album_key')
    const albumId = metaString(message.metadata, 'media_album_id')
    if (!albumKey || !albumId) continue
    const current = groups.get(albumKey) ?? {
      albumKey,
      albumId,
      messages: [],
      attachmentCount: 0,
      occurredAt: message.occurred_at ?? message.projected_at ?? null,
    }
    current.messages.push(message)
    current.attachmentCount += telegramMessageAttachmentHints(message).length
    const occurredAt = message.occurred_at ?? message.projected_at ?? null
    if (occurredAt && (!current.occurredAt || occurredAt < current.occurredAt)) {
      current.occurredAt = occurredAt
    }
    groups.set(albumKey, current)
  }

  return Array.from(groups.values())
    .filter((group) => group.messages.length > 1 || group.attachmentCount > 1)
    .sort((left, right) => (right.occurredAt ?? '').localeCompare(left.occurredAt ?? ''))
}

export function telegramVoiceAttachmentHintsForMessages(
  messages: TelegramMessage[]
): ReturnType<typeof telegramMessageAttachmentHints> {
  return telegramAttachmentHintsForMessages(messages).filter((hint) => hint.kind === 'voice' || hint.kind === 'audio')
}

export function mergeTelegramAttachmentHints(
  galleryItems: TelegramMediaItem[],
  fileHints: TelegramAttachmentHint[]
): TelegramAttachmentHint[] {
  const merged = new Map<string, TelegramAttachmentHint>()
  for (const item of galleryItems) {
    const hint = telegramAttachmentHintFromMediaItem(item)
    merged.set(`${hint.messageId}:${hint.fileName}`, hint)
  }
  for (const hint of fileHints) {
    const key = `${hint.messageId}:${hint.fileName}`
    const current = merged.get(key)
    merged.set(key, {
      ...current,
      ...hint,
      kind: hint.kind === 'file' && current?.kind ? current.kind : hint.kind,
      providerAttachmentId: hint.providerAttachmentId || current?.providerAttachmentId || '',
      providerMessageId: hint.providerMessageId ?? current?.providerMessageId ?? null,
      tdlibFileId: hint.tdlibFileId ?? current?.tdlibFileId ?? null,
      localPath: hint.localPath ?? current?.localPath ?? null,
      downloadState: hint.downloadState === 'unknown' && current?.downloadState ? current.downloadState : hint.downloadState,
    })
  }
  return Array.from(merged.values())
}

export function telegramLinkHintsForMessages(
  messages: TelegramMessage[]
): Array<{ url: string; label: string; occurredAt: string | null }> {
  const hints: Array<{ url: string; label: string; occurredAt: string | null }> = []
  const urlPattern = /https?:\/\/[^\s<>"']+/g
  for (const message of messages) {
    const matches = message.text?.match(urlPattern)
    if (matches) {
      for (const url of matches) {
        hints.push({
          url,
          label: url.length > 60 ? url.slice(0, 60) + '...' : url,
          occurredAt: message.occurred_at ?? message.projected_at ?? null
        })
      }
    }
  }
  return hints
}

export function telegramPinnedMessages(messages: TelegramMessage[]): TelegramMessage[] {
  return messages.filter(
    (m) => metaBoolean(m.metadata, 'is_pinned') || metaBoolean(m.metadata, 'pinned')
  )
}

export function telegramMessageTime(message: TelegramMessage): string {
  const date = message.occurred_at ?? message.projected_at
  if (!date) return ''
  try {
    return new Date(date).toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch {
    return ''
  }
}

export function telegramChatMentionCount(chat: TelegramChat, _messages: TelegramMessage[]): number {
  return telegramChatMentionCountValue(chat)
}

// --- Pinia Store ---

export interface TelegramManualSendForm {
  text: string
}

export const useTelegramStore = defineStore('telegram-ui', () => {
  // Selection state
  const selectedTelegramChatId = ref('')
  const activeTelegramFilter = ref<TelegramChatFilter>('all')
  const activeTelegramGroupFilter = ref('local:all')
  const activeThreadTab = ref<TelegramThreadTab>('messages')
  const activeRailTab = ref<TelegramRailTab>('context')
  const focusedTelegramMessage = ref<TelegramMessage | null>(null)

  // UI state
  const telegramError = ref('')
  const telegramActionMessage = ref('')
  const isTelegramLoading = ref(false)
  const isTelegramActionSubmitting = ref(false)
  const isTelegramHistorySyncing = ref(false)
  const telegramSearchQuery = ref('')
  const isTelegramFiltersMenuOpen = ref(false)
  const isTelegramNewMenuOpen = ref(false)
  const isTelegramInspectorOpen = ref(false)

  // Form state
  const telegramManualSendText = ref('')

  const isTelegramBusy = computed(() =>
    isTelegramActionSubmitting.value || isTelegramHistorySyncing.value
  )

  // Actions
  function selectTelegramChat(chat: TelegramChat) {
    selectedTelegramChatId.value = chat.provider_chat_id
    activeThreadTab.value = 'messages'
    activeRailTab.value = 'context'
    focusedTelegramMessage.value = null
  }

  function selectTelegramFilter(filter: TelegramChatFilter) {
    activeTelegramFilter.value = filter
    closeTelegramMenus()
  }

  function selectTelegramGroupFilter(filter: TelegramChatGroupFilter) {
    activeTelegramGroupFilter.value = filter.id
    closeTelegramMenus()
  }

  function openTelegramInspector(tab: TelegramRailTab = activeRailTab.value) {
    activeRailTab.value = tab
    isTelegramInspectorOpen.value = true
  }

  function closeTelegramInspector() {
    isTelegramInspectorOpen.value = false
  }

  function toggleTelegramInspector() {
    if (isTelegramInspectorOpen.value) {
      closeTelegramInspector()
    } else {
      openTelegramInspector()
    }
  }

  function toggleTelegramFiltersMenu() {
    isTelegramFiltersMenuOpen.value = !isTelegramFiltersMenuOpen.value
    isTelegramNewMenuOpen.value = false
  }

  function toggleTelegramNewMenu() {
    isTelegramNewMenuOpen.value = !isTelegramNewMenuOpen.value
    isTelegramFiltersMenuOpen.value = false
  }

  function closeTelegramMenus() {
    isTelegramFiltersMenuOpen.value = false
    isTelegramNewMenuOpen.value = false
  }

  function setTelegramLoading(loading: boolean) {
    isTelegramLoading.value = loading
  }

  function setTelegramActionSubmitting(submitting: boolean) {
    isTelegramActionSubmitting.value = submitting
  }

  function setTelegramHistorySyncing(syncing: boolean) {
    isTelegramHistorySyncing.value = syncing
  }

  function setTelegramError(error: string) {
    telegramError.value = error
  }

  function setTelegramActionMessage(message: string) {
    telegramActionMessage.value = message
  }

  function resetSendForm() {
    telegramManualSendText.value = ''
  }

  function setTelegramManualSendText(value: string) {
    telegramManualSendText.value = value
  }

  function focusTelegramMessage(message: TelegramMessage) {
    focusedTelegramMessage.value = message
    activeThreadTab.value = 'messages'
  }

  function clearTelegramFocusedMessage() {
    focusedTelegramMessage.value = null
  }

  return {
    // State
    selectedTelegramChatId,
    activeTelegramFilter,
    activeTelegramGroupFilter,
    activeThreadTab,
    activeRailTab,
    focusedTelegramMessage,
    telegramError,
    telegramActionMessage,
    isTelegramLoading,
    isTelegramActionSubmitting,
    isTelegramHistorySyncing,
    telegramSearchQuery,
    isTelegramFiltersMenuOpen,
    isTelegramNewMenuOpen,
    isTelegramInspectorOpen,
    telegramManualSendText,
    isTelegramBusy,
    // Actions
    selectTelegramChat,
    selectTelegramFilter,
    selectTelegramGroupFilter,
    openTelegramInspector,
    closeTelegramInspector,
    toggleTelegramInspector,
    toggleTelegramFiltersMenu,
    toggleTelegramNewMenu,
    closeTelegramMenus,
    setTelegramLoading,
    setTelegramActionSubmitting,
    setTelegramHistorySyncing,
    setTelegramError,
    setTelegramActionMessage,
    resetSendForm,
    setTelegramManualSendText,
    focusTelegramMessage,
    clearTelegramFocusedMessage
  }
})
