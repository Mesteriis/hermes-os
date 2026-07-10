import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('realtime cache patch handling', () => {
  it('patches cached folder-message lists for moved message realtime events', () => {
    const sourceKey = ['communications-folder-messages', 'folder-old']
    const targetKey = ['communications-folder-messages', 'folder-new']
    const movedMessage = {
      folder_id: 'folder-new',
      message_id: 'msg-1',
      account_id: 'account-1',
      subject: 'Project update',
      sender: 'sender@example.com',
      occurred_at: '2026-06-15T09:00:00Z',
      projected_at: '2026-06-15T09:01:00Z',
      workflow_state: 'new',
      local_state: 'active',
      added_at: '2026-06-15T10:00:00Z',
      attachment_count: 0
    }
    const sourceData = {
      pages: [
        {
          items: [{ ...movedMessage, folder_id: 'folder-old' }],
          next_cursor: null,
          has_more: false
        }
      ],
      pageParams: [null]
    }
    const targetData = {
      pages: [
        {
          items: [],
          next_cursor: null,
          has_more: false
        }
      ],
      pageParams: [null]
    }
    const setQueryData = vi.fn()
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([
        [sourceKey, sourceData],
        [targetKey, targetData]
      ]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: '58',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.folder_message.moved',
            payload: {
              operation: 'move',
              folder_id: 'folder-new',
              message_id: 'msg-1',
              message: movedMessage
            }
          }
        })
      },
      queryClient
    )

    expect(setQueryData).toHaveBeenCalledWith(sourceKey, {
      ...sourceData,
      pages: [{ ...sourceData.pages[0], items: [] }]
    })
    expect(setQueryData).toHaveBeenCalledWith(targetKey, {
      ...targetData,
      pages: [{ ...targetData.pages[0], items: [movedMessage] }]
    })
  })

  it('patches cached saved-search lists for saved search realtime events', () => {
    const savedSearchKey = ['communications-saved-searches', false, undefined]
    const smartFolderKey = ['communications-saved-searches', true, undefined]
    const savedSearch = {
      saved_search_id: 'search-1',
      name: 'Invoices',
      description: null,
      account_id: null,
      query: 'invoice',
      workflow_state: null,
      local_state: 'active',
      channel_kind: 'email',
      is_smart_folder: false,
      sort_order: 1000,
      message_count: 2,
      created_at: '2026-06-15T10:00:00Z',
      updated_at: '2026-06-15T10:00:00Z'
    }
    const smartFolder = {
      ...savedSearch,
      saved_search_id: 'smart-1',
      name: 'Needs Action',
      query: 'state:needs_action',
      is_smart_folder: true
    }
    const savedSearchData = {
      pages: [{ items: [savedSearch], next_cursor: null, has_more: false }],
      pageParams: [null]
    }
    const smartFolderData = {
      pages: [{ items: [smartFolder], next_cursor: null, has_more: false }],
      pageParams: [null]
    }
    const setQueryData = vi.fn()
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([
        [savedSearchKey, savedSearchData],
        [smartFolderKey, smartFolderData]
      ]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: '56',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.saved_search.updated',
            payload: {
              ...savedSearch,
              name: 'Paid invoices',
              message_count: 3,
              updated_at: '2026-06-15T10:01:00Z'
            }
          }
        })
      },
      queryClient
    )

    expect(setQueryData).toHaveBeenCalledOnce()
    expect(setQueryData).toHaveBeenCalledWith(savedSearchKey, {
      ...savedSearchData,
      pages: [
        {
          ...savedSearchData.pages[0],
          items: [
            {
              ...savedSearch,
              name: 'Paid invoices',
              message_count: 3,
              updated_at: '2026-06-15T10:01:00Z'
            }
          ]
        }
      ]
    })
  })

  it('patches cached sync statuses for sync progress realtime events', () => {
    const syncKey = ['communications', 'mail', 'sync-statuses']
    const statuses = [
      {
        account_id: 'account-1',
        status: 'running',
        phase: 'fetch',
        progress_mode: 'determinate',
        progress_percent: 10,
        processed_messages: 5,
        estimated_total_messages: 50,
        current_batch_size: 10,
        last_started_at: '2026-06-15T10:00:00Z',
        last_updated_at: '2026-06-15T10:00:05Z',
        last_completed_at: null,
        next_run_at: null,
        last_error_code: null,
        last_error_message: null,
        last_fetched_messages: 5,
        last_projected_messages: 4,
        last_upserted_personas: 1,
        last_upserted_organizations: 0
      }
    ]
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(statuses) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([[syncKey, statuses]]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: '57',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.sync.progress',
            payload: {
              account_id: 'account-1',
              status: 'running',
              phase: 'project',
              progress_mode: 'determinate',
              progress_percent: 60,
              processed_messages: 30,
              estimated_total_messages: 50,
              current_batch_size: 10,
              fetched_messages: 30,
              projected_messages: 24,
              upserted_personas: 5,
              upserted_organizations: 2,
              next_run_at: null
            }
          }
        })
      },
      queryClient
    )

    expect(setQueryData).toHaveBeenCalledWith(syncKey, expect.any(Array))
    expect(setQueryData.mock.results[0]?.value[0]).toMatchObject({
      account_id: 'account-1',
      phase: 'project',
      progress_percent: 60,
      processed_messages: 30,
      last_updated_at: expect.any(String),
      last_fetched_messages: 30,
      last_projected_messages: 24,
      last_upserted_personas: 5,
      last_upserted_organizations: 2
    })
  })

  it('invalidates only draft queries for mail draft events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: '46',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.draft.updated'
          }
        })
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledOnce()
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-drafts']
    })
  })

  it('falls back to broad invalidation for unknown canonical events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: '44',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.unknown.changed'
          }
        })
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'mail', 'sync-statuses']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-attachment-search']
    })
  })

  it('ignores heartbeat events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent({ id: '', event: 'heartbeat', data: '{}' }, queryClient)

    expect(queryClient.invalidateQueries).not.toHaveBeenCalled()
  })
})
