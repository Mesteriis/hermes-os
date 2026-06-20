import { describe, expect, it } from 'vitest'
import type { TelegramChat } from './types/telegram'
import { telegramChatFolderIds } from './folderMembership'

function chat(metadata: TelegramChat['metadata']): TelegramChat {
  return {
    telegram_chat_id: 'tgchat-1',
    account_id: 'acc-1',
    provider_chat_id: 'chat-1',
    chat_kind: 'group',
    title: 'Project Room',
    username: null,
    sync_state: 'synced',
    last_message_at: null,
    metadata,
    created_at: '2026-06-16T10:00:00Z',
    updated_at: '2026-06-16T10:00:00Z',
  }
}

describe('telegramChatFolderIds', () => {
  it('prefers tdlib chat positions when available', () => {
    expect(
      telegramChatFolderIds(
        chat({
          tdlib_chat_positions: {
            folder_ids: [7, 11],
          },
          provider_folder_ids: [9],
          provider_folder_id: 9,
        })
      )
    ).toEqual([7, 11])
  })

  it('falls back to provider_folder_ids metadata when tdlib positions are absent', () => {
    expect(
      telegramChatFolderIds(
        chat({
          provider_folder_ids: [7, 11],
          provider_folder_id: 7,
        })
      )
    ).toEqual([7, 11])
  })

  it('falls back to singular provider_folder_id for legacy metadata', () => {
    expect(
      telegramChatFolderIds(
        chat({
          provider_folder_id: 7,
        })
      )
    ).toEqual([7])
  })
})
