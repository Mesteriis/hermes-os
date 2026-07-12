import { describe, expect, it } from 'vitest'
import {
  telegramChatNeedsRead,
  telegramForwardTargets,
  telegramMessageSendStatus,
  telegramProviderFolders,
  telegramTdlibMessageId,
  latestTelegramInboundMessageId,
} from './telegramWorkspacePresentation'

describe('telegram workspace presentation', () => {
  it('excludes the current dialog from forward targets', () => {
    const targets = telegramForwardTargets([
      { provider_chat_id: 'chat-current' },
      { provider_chat_id: 'chat-target' },
    ] as never, 'chat-current')

    expect(targets).toEqual([{ provider_chat_id: 'chat-target' }])
  })

  it('keeps only provider-backed folders for folder commands', () => {
    const folders = telegramProviderFolders([
      { id: 'local:all', provider_folder_id: null },
      { id: 'telegram:7', provider_folder_id: 7 },
    ] as never)

    expect(folders).toEqual([{ id: 'telegram:7', provider_folder_id: 7 }])
  })

  it('marks a chat as eligible for delayed read only when provider state is unread', () => {
    expect(telegramChatNeedsRead({ metadata: { unread_count: 2 } } as never)).toBe(true)
    expect(telegramChatNeedsRead({ metadata: { is_marked_as_unread: true } } as never)).toBe(true)
    expect(telegramChatNeedsRead({ metadata: { provider_unread_count: 0 } } as never)).toBe(false)
  })

  it('selects only received messages as the provider read marker', () => {
    expect(latestTelegramInboundMessageId([
      { delivery_state: 'send_blocked', provider_message_id: 'blocked' },
      { delivery_state: 'sent', provider_message_id: 'sent' },
      { delivery_state: 'received', provider_message_id: 'received' },
    ])).toBe('received')
  })

  it('describes the actual Telegram send outcome', () => {
    expect(telegramMessageSendStatus('sent')).toBe('Telegram message sent.')
    expect(telegramMessageSendStatus('queued')).toBe('Telegram message queued.')
    expect(telegramMessageSendStatus('failed')).toBe('Telegram message failed.')
    expect(telegramMessageSendStatus('send_failed')).toBe('Telegram message failed.')
  })

  it('extracts a TDLib message id from the canonical provider locator', () => {
    expect(telegramTdlibMessageId('-1003320156340:101705580544')).toBe(101705580544)
    expect(telegramTdlibMessageId('not-a-message')).toBeUndefined()
  })
})
