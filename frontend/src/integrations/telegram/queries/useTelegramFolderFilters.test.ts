import { describe, expect, it } from 'vitest'
import { resolveTelegramGroupFilters } from './useTelegramFolderFilters'
import type { TelegramChat, TelegramChatGroupFilter } from '../types/telegram'

function chat(overrides: Partial<TelegramChat> = {}): TelegramChat {
  return {
    telegram_chat_id: overrides.telegram_chat_id ?? 'tgchat-1',
    account_id: overrides.account_id ?? 'acc-1',
    provider_chat_id: overrides.provider_chat_id ?? 'chat-1',
    chat_kind: overrides.chat_kind ?? 'group',
    title: overrides.title ?? 'Project Room',
    username: overrides.username ?? null,
    sync_state: overrides.sync_state ?? 'synced',
    last_message_at: overrides.last_message_at ?? null,
    metadata: overrides.metadata ?? {},
    created_at: overrides.created_at ?? '2026-06-16T10:00:00Z',
    updated_at: overrides.updated_at ?? '2026-06-16T10:00:00Z',
  }
}

describe('useTelegramFolderFilters', () => {
  it('prefers projection-backed server filters when available', () => {
    const chats = [
      chat({ metadata: { folder_name: 'Local Work' } }),
      chat({ telegram_chat_id: 'tgchat-2', provider_chat_id: 'chat-2', metadata: { folder_name: 'Local Work' } }),
    ]
    const serverFilters: TelegramChatGroupFilter[] = [
      { id: 'local:all', label: 'All', source: 'local', count: 2, icon: 'tabler:message' },
      { id: 'folder:Server Work', label: 'Server Work', source: 'telegram', count: 2, icon: 'tabler:folder' },
    ]

    expect(resolveTelegramGroupFilters(chats, serverFilters)).toEqual(serverFilters)
  })

  it('falls back to local chat-derived folder filters when server filters are unavailable', () => {
    const chats = [
      chat({ metadata: { folder_name: 'Work' } }),
      chat({ telegram_chat_id: 'tgchat-2', provider_chat_id: 'chat-2', metadata: { folder_name: 'Archive' } }),
    ]

    expect(resolveTelegramGroupFilters(chats, null)).toEqual([
      { id: 'local:all', label: 'All', source: 'local', count: 2, icon: 'tabler:message' },
      { id: 'folder:Work', label: 'Work', source: 'telegram', count: 1, icon: 'tabler:folder' },
      { id: 'folder:Archive', label: 'Archive', source: 'telegram', count: 1, icon: 'tabler:folder' },
    ])
  })

  it('derives fallback folder filters from provider-synced folder label arrays', () => {
    const chats = [
      chat({ metadata: { folder_labels: ['Projects', 'Pinned'] } }),
      chat({ telegram_chat_id: 'tgchat-2', provider_chat_id: 'chat-2', metadata: { folder_labels: ['Projects'] } }),
    ]

    expect(resolveTelegramGroupFilters(chats, null)).toEqual([
      { id: 'local:all', label: 'All', source: 'local', count: 2, icon: 'tabler:message' },
      { id: 'folder:Projects', label: 'Projects', source: 'telegram', count: 2, icon: 'tabler:folder' },
      { id: 'folder:Pinned', label: 'Pinned', source: 'telegram', count: 1, icon: 'tabler:folder' },
    ])
  })
})
