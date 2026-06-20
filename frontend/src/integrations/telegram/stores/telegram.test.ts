import { describe, expect, it } from 'vitest'
import {
  mergeTelegramAttachmentHints,
  telegramChatLastReadInboxProviderMessageId,
  telegramChatPreview,
  telegramChatTypingLabel,
  telegramFilterTabs,
  telegramMediaAlbumGroupsForMessages,
  telegramMessageAttachmentHints,
} from './telegram'

describe('telegram chat typing projection helpers', () => {
  it('keeps Telegram chat filter tabs centralized for the page shell', () => {
    expect(telegramFilterTabs((key) => key).map((tab) => tab.id)).toEqual([
      'all',
      'unread',
      'mentions',
      'pinned',
      'projects',
      'bots',
      'archived'
    ])
  })

  it('surfaces active typing metadata ahead of the message preview', () => {
    const chat = {
      telegram_chat_id: 'tgchat-1',
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      chat_kind: 'private' as const,
      title: 'Chat',
      username: null,
      sync_state: 'synced' as const,
      last_message_at: null,
      metadata: { active_typing: { sender_id: 'user:777', is_active: true } },
      created_at: '2026-06-16T10:00:00Z',
      updated_at: '2026-06-16T10:00:00Z',
    }

    expect(telegramChatTypingLabel(chat)).toBe('user:777 typing...')
    expect(telegramChatPreview(chat, [])).toBe('user:777 typing...')
  })

  it('hides stale typing metadata after its projected expiry time', () => {
    const chat = {
      telegram_chat_id: 'tgchat-1',
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      chat_kind: 'private' as const,
      title: 'Chat',
      username: null,
      sync_state: 'synced' as const,
      last_message_at: null,
      metadata: {
        active_typing: {
          sender_id: 'user:777',
          is_active: true,
          expires_at: '2026-06-16T10:00:07.000Z',
        },
      },
      created_at: '2026-06-16T10:00:00Z',
      updated_at: '2026-06-16T10:00:00Z',
    }

    expect(telegramChatTypingLabel(chat, Date.parse('2026-06-16T10:00:06.999Z'))).toBe('user:777 typing...')
    expect(telegramChatTypingLabel(chat, Date.parse('2026-06-16T10:00:07.000Z'))).toBe('')
  })

  it('reads provider last-read inbox progress from projected chat metadata', () => {
    const chat = {
      telegram_chat_id: 'tgchat-1',
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      chat_kind: 'private' as const,
      title: 'Chat',
      username: null,
      sync_state: 'synced' as const,
      last_message_at: null,
      metadata: { last_read_inbox_provider_message_id: '777' },
      created_at: '2026-06-16T10:00:00Z',
      updated_at: '2026-06-16T10:00:00Z',
    }

    expect(telegramChatLastReadInboxProviderMessageId(chat)).toBe('777')
  })
})

describe('telegram media attachment helpers', () => {
  it('preserves query-backed TDLib and local-path metadata when merging media search hits with loaded file hints', () => {
    const merged = mergeTelegramAttachmentHints(
      [{
        message_id: 'msg-1',
        provider_message_id: '42',
        provider_chat_id: 'chat-1',
        file_name: 'invoice.pdf',
        kind: 'document',
        mime_type: 'application/pdf',
        size_bytes: 2048,
        occurred_at: '2026-06-16T10:00:00Z',
        download_state: 'downloaded',
        tdlib_file_id: 9001,
        provider_attachment_id: 'attachment-1',
        local_path: '/tmp/hermes/invoice.pdf',
      }],
      [{
        id: 'msg-1:invoice.pdf',
        kind: 'file',
        fileName: 'invoice.pdf',
        mimeType: 'application/pdf',
        sizeBytes: 2048,
        tdlibFileId: null,
        providerAttachmentId: '',
        downloadState: 'unknown',
        localPath: null,
        messageId: 'msg-1',
        providerMessageId: null,
      }]
    )

    expect(merged).toHaveLength(1)
    expect(merged[0]).toMatchObject({
      fileName: 'invoice.pdf',
      kind: 'document',
      tdlibFileId: 9001,
      providerAttachmentId: 'attachment-1',
      localPath: '/tmp/hermes/invoice.pdf',
      downloadState: 'downloaded',
      providerMessageId: '42',
    })
  })

  it('derives sticker, animation and video note attachment hints from projected metadata', () => {
    const hints = telegramMessageAttachmentHints({
      message_id: 'msg-media',
      raw_record_id: 'raw-media',
      account_id: 'account-1',
      provider_message_id: 'chat-1:77',
      provider_chat_id: 'chat-1',
      chat_title: 'Media chat',
      sender: 'sender-1',
      sender_display_name: 'Sender',
      text: '',
      occurred_at: '2026-06-16T10:00:00Z',
      projected_at: '2026-06-16T10:00:01Z',
      channel_kind: 'telegram_user',
      delivery_state: 'received',
      metadata: {
        attachments: [
          { attachment_type: 'sticker', filename: 'ok.webp', content_type: 'image/webp', tdlib_file_id: 701, download_state: 'remote' },
          { attachment_type: 'animation', filename: 'loop.mp4', content_type: 'video/mp4', tdlib_file_id: 702, download_state: 'remote' },
          { attachment_type: 'video_note', filename: 'video-note.mp4', content_type: 'video/mp4', tdlib_file_id: 703, download_state: 'remote' },
        ],
      },
    })

    expect(hints.map((hint) => hint.kind)).toEqual(['sticker', 'animation', 'video_note'])
    expect(hints.map((hint) => hint.tdlibFileId)).toEqual([701, 702, 703])
  })

  it('groups projected media album messages by album key', () => {
    const groups = telegramMediaAlbumGroupsForMessages([
      {
        message_id: 'msg-1',
        raw_record_id: 'raw-1',
        account_id: 'account-1',
        provider_message_id: 'chat-1:10',
        provider_chat_id: 'chat-1',
        chat_title: 'Media chat',
        sender: 'sender-1',
        sender_display_name: 'Sender',
        text: '',
        occurred_at: '2026-06-16T10:00:00Z',
        projected_at: '2026-06-16T10:00:01Z',
        channel_kind: 'telegram_user',
        delivery_state: 'received',
        metadata: {
          media_album_id: 'album-1',
          media_album_key: 'chat-1:album-1',
          attachments: [{ attachment_type: 'photo', filename: 'a.jpg' }],
        },
      },
      {
        message_id: 'msg-2',
        raw_record_id: 'raw-2',
        account_id: 'account-1',
        provider_message_id: 'chat-1:11',
        provider_chat_id: 'chat-1',
        chat_title: 'Media chat',
        sender: 'sender-1',
        sender_display_name: 'Sender',
        text: '',
        occurred_at: '2026-06-16T10:00:02Z',
        projected_at: '2026-06-16T10:00:03Z',
        channel_kind: 'telegram_user',
        delivery_state: 'received',
        metadata: {
          media_album_id: 'album-1',
          media_album_key: 'chat-1:album-1',
          attachments: [{ attachment_type: 'photo', filename: 'b.jpg' }],
        },
      },
    ])

    expect(groups).toHaveLength(1)
    expect(groups[0]).toMatchObject({
      albumId: 'album-1',
      albumKey: 'chat-1:album-1',
      attachmentCount: 2,
    })
    expect(groups[0].messages.map((message) => message.message_id)).toEqual(['msg-1', 'msg-2'])
  })
})
