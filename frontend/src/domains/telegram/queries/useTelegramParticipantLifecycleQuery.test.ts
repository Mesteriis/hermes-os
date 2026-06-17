import { QueryClient } from '@tanstack/vue-query'
import { describe, expect, it } from 'vitest'
import type { TelegramProviderWriteCommand } from '../types/telegram'
import { primeTelegramParticipantLifecycleCommandCache } from './useTelegramParticipantLifecycleQuery'

function queryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  })
}

describe('telegram participant lifecycle command cache priming', () => {
  it('inserts join command into matching account command caches before realtime reconciliation', () => {
    const client = queryClient()
    const accountCommandsKey = ['telegram', 'commands', 'account-1', 10] as const
    const otherAccountCommandsKey = ['telegram', 'commands', 'account-2', 10] as const

    client.setQueryData<TelegramProviderWriteCommand[]>(accountCommandsKey, [])
    client.setQueryData<TelegramProviderWriteCommand[]>(otherAccountCommandsKey, [])

    primeTelegramParticipantLifecycleCommandCache(client, 'account-1', {
      telegram_chat_id: 'tgchat-1',
      provider_chat_id: 'chat-1',
      action: 'join',
      status: 'queued',
      command_id: 'cmd-join-1',
    })

    expect(client.getQueryData<TelegramProviderWriteCommand[]>(accountCommandsKey)).toMatchObject([
      {
        command_id: 'cmd-join-1',
        account_id: 'account-1',
        command_kind: 'join',
        provider_chat_id: 'chat-1',
        status: 'queued',
      },
    ])
    expect(client.getQueryData<TelegramProviderWriteCommand[]>(otherAccountCommandsKey)).toEqual([])
  })

  it('inserts leave command with current chat target metadata before reconciliation arrives', () => {
    const client = queryClient()
    const accountCommandsKey = ['telegram', 'commands', 'account-1', 10] as const

    client.setQueryData<TelegramProviderWriteCommand[]>(accountCommandsKey, [])

    primeTelegramParticipantLifecycleCommandCache(client, 'account-1', {
      telegram_chat_id: 'tgchat-9',
      provider_chat_id: 'chat-9',
      action: 'leave',
      status: 'queued',
      command_id: 'cmd-leave-9',
    })

    expect(client.getQueryData<TelegramProviderWriteCommand[]>(accountCommandsKey)).toMatchObject([
      {
        command_id: 'cmd-leave-9',
        account_id: 'account-1',
        command_kind: 'leave',
        provider_chat_id: 'chat-9',
        target_ref: {
          provider_chat_id: 'chat-9',
          telegram_chat_id: 'tgchat-9',
        },
      },
    ])
  })
})
