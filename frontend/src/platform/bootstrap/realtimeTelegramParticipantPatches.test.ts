import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('telegram participant realtime cache patching', () => {
  it('upserts cached chat members for participant update events across infinite member pages', () => {
    const membersKey = ['telegram', 'chat-members', 'tgchat-1', 50, '', '']
    const existing = {
      pages: [
        {
          items: [
            {
              sender_id: 'user:1',
              sender_display_name: 'Old Member',
              message_count: 0,
              last_message_at: null,
              source: 'tdlib',
              provider_member_id: 'user:1',
              username: null,
              role: 'member',
              status: 'member',
              is_admin: false,
              is_owner: false,
              permissions: {},
              observed_at: null,
            },
          ],
          next_cursor: null,
        },
      ],
      pageParams: [null],
    }
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(existing) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['telegram', 'chat-members'])) {
          return [[membersKey, existing]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-participant-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.participant.updated',
            payload: {
              telegram_chat_id: 'tgchat-1',
              participant: {
                sender_id: 'user:42',
                sender_display_name: 'Owner User',
                provider_member_id: 'user:42',
                source: 'tdlib',
                role: 'owner',
                status: 'creator',
                is_admin: true,
                is_owner: true,
                permissions: { can_invite_users: true },
                observed_at: '2026-06-17T00:00:00Z'
              }
            }
          }
        })
      },
      queryClient
    )

    const patched = setQueryData.mock.results[0]?.value
    const patchedItems = patched.pages[0].items
    expect(patchedItems[0]).toMatchObject({
      provider_member_id: 'user:42',
      sender_display_name: 'Owner User',
      role: 'owner',
      is_owner: true
    })
    expect(patchedItems[1].provider_member_id).toBe('user:1')
  })

  it('removes cached chat members when exhaustive provider absence is observed', () => {
    const membersKey = ['telegram', 'chat-members', 'tgchat-1', 50, '', '']
    const existing = {
      pages: [
        {
          items: [
            {
              sender_id: 'user:42',
              sender_display_name: 'Owner User',
              message_count: 0,
              last_message_at: null,
              source: 'tdlib',
              provider_member_id: 'user:42',
              username: null,
              role: 'owner',
              status: 'creator',
              is_admin: true,
              is_owner: true,
              permissions: {},
              observed_at: null,
            },
          ],
          next_cursor: null,
        },
      ],
      pageParams: [null],
    }
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(existing) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['telegram', 'chat-members'])) {
          return [[membersKey, existing]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-participant-absence-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.participant.updated',
            payload: {
              telegram_chat_id: 'tgchat-1',
              participant: {
                sender_id: 'user:42',
                sender_display_name: 'Owner User',
                provider_member_id: 'user:42',
                source: 'tdlib',
                role: 'owner',
                status: 'absent_exhaustive',
                is_admin: true,
                is_owner: true,
                permissions: {
                  membership_state: 'absent_exhaustive',
                },
                observed_at: '2026-06-17T00:00:00Z'
              }
            }
          }
        })
      },
      queryClient
    )

    const patched = setQueryData.mock.results[0]?.value
    expect(patched.pages[0].items).toEqual([])
  })

  it('removes cached chat members when participant lifecycle becomes inactive', () => {
    const membersKey = ['telegram', 'chat-members', 'tgchat-1', 50, '', '']
    const existing = {
      pages: [
        {
          items: [
            {
              sender_id: 'user:42',
              sender_display_name: 'Former Member',
              message_count: 0,
              last_message_at: null,
              source: 'tdlib',
              provider_member_id: 'user:42',
              username: null,
              role: 'member',
              status: 'member',
              is_admin: false,
              is_owner: false,
              permissions: {},
              observed_at: null,
            },
          ],
          next_cursor: null,
        },
      ],
      pageParams: [null],
    }
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(existing) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['telegram', 'chat-members'])) {
          return [[membersKey, existing]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-participant-left-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.participant.updated',
            payload: {
              telegram_chat_id: 'tgchat-1',
              participant: {
                sender_id: 'user:42',
                sender_display_name: 'Former Member',
                provider_member_id: 'user:42',
                source: 'tdlib',
                role: 'left',
                status: 'left',
                is_admin: false,
                is_owner: false,
                permissions: {
                  membership_state: 'left',
                },
                observed_at: '2026-06-17T00:00:00Z'
              }
            }
          }
        })
      },
      queryClient
    )

    const patched = setQueryData.mock.results[0]?.value
    expect(patched.pages[0].items).toEqual([])
  })
})
