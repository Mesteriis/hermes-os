import { describe, expect, it } from 'vitest'
import type { TelegramMessage } from '@/shared/communications/types/telegram'
import {
  buildTelegramDeleteRequest,
  buildTelegramEditRequest,
  buildTelegramForwardRequest,
  buildTelegramMarkReadRequest,
  buildTelegramPinRequest,
  buildTelegramReactionRequest,
  buildTelegramReactionMutationRequest,
  buildTelegramReplyRequest,
  buildTelegramRestoreRequest,
} from './telegramMessageInspectorActions'

describe('telegram message inspector actions', () => {
  it('builds lifecycle requests from one message provenance source', () => {
    const message = telegramMessage()
    expect(buildTelegramEditRequest(message, 'updated', 'command-1').new_text).toBe('updated')
    expect(buildTelegramDeleteRequest(message, 'command-2')).toMatchObject({
      reason_class: 'deleted_by_owner', actor_class: 'owner', is_provider_delete: true,
    })
    expect(buildTelegramRestoreRequest(message, 'command-3').reason).toBe('owner_requested_restore')
    expect(buildTelegramReactionRequest(message, '  👍  ').reaction_emoji).toBe('👍')
    expect(buildTelegramReactionMutationRequest(message, '👍')).toMatchObject({
      messageId: 'message-1', request: { reaction_emoji: '👍' },
    })
    expect(buildTelegramMarkReadRequest(message).provider_chat_id).toBe('chat-1')
    expect(buildTelegramPinRequest(message)).toEqual({ message_id: 'message-1' })
    expect(buildTelegramReplyRequest(message, 'reply')).toEqual({ message_id: 'message-1', text: 'reply' })
    expect(buildTelegramForwardRequest(message, 'chat-2')).toEqual({ message_id: 'message-1', provider_chat_id: 'chat-2' })
  })
})

function telegramMessage(): TelegramMessage {
  return {
    message_id: 'message-1', account_id: 'account-1', provider_chat_id: 'chat-1',
    provider_message_id: 'provider-message-1', text: 'Original',
  } as TelegramMessage
}
