import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type {
  TelegramChat,
  TelegramMessage,
  TelegramCapabilitiesResponse,
  TelegramRuntimeStatus,
  TelegramChatFilter,
  TelegramChatGroupFilter,
  TelegramRailTab,
  TelegramThreadTab,
  TelegramChatFilterCount,
  TelegramAttachmentHint
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

export function telegramChatIsPinned(chat: TelegramChat): boolean {
  return metaBoolean(chat.metadata, 'is_pinned') || metaBoolean(chat.metadata, 'pinned')
}

export function telegramChatIsProject(chat: TelegramChat): boolean {
  return metaBoolean(chat.metadata, 'is_project')
}

export function telegramChatIsBot(chat: TelegramChat): boolean {
  return chat.chat_kind === 'bot'
}

export function telegramChatPreview(chat: TelegramChat, messages: TelegramMessage[]): string {
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
  const mentions = chats.filter((c) => metaNumber(c.metadata, 'mention_count') > 0).length
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

export function telegramChatGroupFilters(chats: TelegramChat[]): TelegramChatGroupFilter[] {
  const groups: TelegramChatGroupFilter[] = [
    { id: 'local:all', label: 'All', icon: 'tabler:message', source: 'local', count: chats.length }
  ]
  const folders = new Map<string, { name: string; count: number }>()
  for (const chat of chats) {
    const folderName = metaString(chat.metadata, 'folder_name')
    if (folderName) {
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
    return chats.filter((c) => metaString(c.metadata, 'folder_name') === folderName)
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
        return metaNumber(chat.metadata, 'mention_count') > 0
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
    messageId: message.message_id
  }))
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
  return messages.filter((m) => metaBoolean(m.metadata, 'is_pinned'))
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
  return metaNumber(chat.metadata, 'mention_count')
}

// --- Pinia Store ---

export interface TelegramManualSendForm {
  text: string
}

export const useTelegramStore = defineStore('telegram-ui', () => {
  // Data state
  const telegramChats = ref<TelegramChat[]>([])
  const telegramMessages = ref<TelegramMessage[]>([])
  const telegramCapabilities = ref<TelegramCapabilitiesResponse | null>(null)
  const telegramRuntimeStatuses = ref<Record<string, TelegramRuntimeStatus>>({})

  // Selection state
  const selectedTelegramChatId = ref('')
  const activeTelegramFilter = ref<TelegramChatFilter>('all')
  const activeTelegramGroupFilter = ref('local:all')
  const activeThreadTab = ref<TelegramThreadTab>('messages')
  const activeRailTab = ref<TelegramRailTab>('context')

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
  const telegramManualSendForm = ref<TelegramManualSendForm>({ text: '' })

  // Derived
  const selectedTelegramChat = computed(() =>
    telegramChats.value.find((chat) => chat.provider_chat_id === selectedTelegramChatId.value) ??
    telegramChats.value[0] ??
    null
  )

  const selectedTelegramMessages = computed(() => {
    if (!selectedTelegramChat.value) return telegramMessages.value
    return telegramMessagesChronological(
      telegramMessages.value.filter(
        (message) => message.provider_chat_id === selectedTelegramChat.value!.provider_chat_id
      )
    )
  })

  const chatFilterCounts = computed<TelegramChatFilterCount[]>(() =>
    telegramChatFilterCounts(telegramChats.value, telegramMessages.value)
  )

  const chatGroupFilters = computed<TelegramChatGroupFilter[]>(() =>
    telegramChatGroupFilters(telegramChats.value)
  )

  const filteredTelegramChats = computed(() =>
    filterTelegramChats(
      filterTelegramChatsByGroup(telegramChats.value, activeTelegramGroupFilter.value),
      telegramMessages.value,
      telegramSearchQuery.value,
      activeTelegramFilter.value
    )
  )

  const selectedTelegramRuntimeStatus = computed(() => {
    if (!selectedTelegramChat.value) return null
    return telegramRuntimeStatuses.value[selectedTelegramChat.value.account_id] ?? null
  })

  const isTelegramBusy = computed(() =>
    isTelegramActionSubmitting.value || isTelegramHistorySyncing.value
  )

  // Actions
  function setTelegramData(data: {
    chats: TelegramChat[]
    messages: TelegramMessage[]
    capabilities: TelegramCapabilitiesResponse | null
    runtimeStatuses: Record<string, TelegramRuntimeStatus>
    selectedChatId: string
    error: string
  }) {
    telegramChats.value = data.chats
    telegramMessages.value = data.messages
    telegramCapabilities.value = data.capabilities
    telegramRuntimeStatuses.value = data.runtimeStatuses
    selectedTelegramChatId.value = data.selectedChatId
    telegramError.value = data.error
  }

  function selectTelegramChat(chat: TelegramChat) {
    selectedTelegramChatId.value = chat.provider_chat_id
    activeThreadTab.value = 'messages'
    activeRailTab.value = 'context'
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

  function updateRuntimeStatus(accountId: string, status: TelegramRuntimeStatus) {
    telegramRuntimeStatuses.value = {
      ...telegramRuntimeStatuses.value,
      [accountId]: status
    }
  }

  function resetSendForm() {
    telegramManualSendForm.value = { text: '' }
  }

  return {
    // State
    telegramChats,
    telegramMessages,
    telegramCapabilities,
    telegramRuntimeStatuses,
    selectedTelegramChatId,
    activeTelegramFilter,
    activeTelegramGroupFilter,
    activeThreadTab,
    activeRailTab,
    telegramError,
    telegramActionMessage,
    isTelegramLoading,
    isTelegramActionSubmitting,
    isTelegramHistorySyncing,
    telegramSearchQuery,
    isTelegramFiltersMenuOpen,
    isTelegramNewMenuOpen,
    isTelegramInspectorOpen,
    telegramManualSendForm,
    // Derived
    selectedTelegramChat,
    selectedTelegramMessages,
    chatFilterCounts,
    chatGroupFilters,
    filteredTelegramChats,
    selectedTelegramRuntimeStatus,
    isTelegramBusy,
    // Actions
    setTelegramData,
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
    updateRuntimeStatus,
    resetSendForm
  }
})
