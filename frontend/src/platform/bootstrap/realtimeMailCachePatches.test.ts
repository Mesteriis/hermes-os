import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('realtime bootstrap mail cache patches', () => {
  it('patches cached outbox metadata for delivery status and read receipt events', () => {
    const outboxKey = ['communications-outbox', undefined, undefined]
    const outboxItems = {
      pages: [
        {
          items: [
            {
              outbox_id: 'outbox-1',
              account_id: 'account-1',
              status: 'sent',
              provider_message_id: 'provider-1',
              last_error: null,
              send_attempts: 1,
              scheduled_send_at: null,
              undo_deadline_at: null,
              sent_at: '2026-06-15T10:00:00Z',
              metadata: {}
            }
          ],
          next_cursor: null,
          has_more: false
        }
      ],
      pageParams: [null]
    }
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(outboxItems) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([[outboxKey, outboxItems]]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: '50',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.outbox.delivery_status_changed',
            payload: {
              outbox_id: 'outbox-1',
              delivery_status: 'delivered',
              source_kind: 'provider_runtime',
              recorded_at: '2026-06-15T10:01:00Z'
            }
          }
        })
      },
      queryClient
    )
    const patchedDeliveryItems = setQueryData.mock.results[0]?.value
    expect(patchedDeliveryItems.pages[0].items[0].metadata.delivery_status).toMatchObject({
      delivery_status: 'delivered',
      source_kind: 'provider_runtime',
      recorded_at: '2026-06-15T10:01:00Z'
    })

    handleRealtimeEvent(
      {
        id: '51',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.read_receipt.recorded',
            payload: {
              outbox_id: 'outbox-1',
              receipt_id: 'receipt-1',
              receipt_kind: 'read',
              read_at: '2026-06-15T10:02:00Z',
              source_kind: 'provider_runtime'
            }
          }
        })
      },
      queryClient
    )

    const patchedReadItems = setQueryData.mock.results[1]?.value
    expect(patchedReadItems.pages[0].items[0].metadata.latest_read_receipt).toMatchObject({
      receipt_id: 'receipt-1',
      receipt_kind: 'read',
      read_at: '2026-06-15T10:02:00Z'
    })
  })

  it('removes cached drafts for draft deleted realtime events', () => {
    const draftsKey = ['communications-drafts', 'account-1']
    const drafts = [
      {
        draft_id: 'draft-1',
        account_id: 'account-1',
        persona_id: null,
        to_recipients: [],
        cc_recipients: [],
        bcc_recipients: [],
        subject: '',
        body_text: '',
        body_html: null,
        in_reply_to: null,
        references: [],
        status: 'draft',
        scheduled_send_at: null,
        send_attempts: 0,
        last_error: null,
        metadata: {},
        created_at: '2026-06-15T10:00:00Z',
        updated_at: '2026-06-15T10:00:00Z'
      },
      {
        draft_id: 'draft-2',
        account_id: 'account-1',
        persona_id: null,
        to_recipients: [],
        cc_recipients: [],
        bcc_recipients: [],
        subject: '',
        body_text: '',
        body_html: null,
        in_reply_to: null,
        references: [],
        status: 'draft',
        scheduled_send_at: null,
        send_attempts: 0,
        last_error: null,
        metadata: {},
        created_at: '2026-06-15T10:00:00Z',
        updated_at: '2026-06-15T10:00:00Z'
      }
    ]
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(drafts) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([[draftsKey, drafts]]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: '53',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.draft.deleted',
            payload: {
              draft_id: 'draft-1',
              account_id: 'account-1'
            }
          }
        })
      },
      queryClient
    )

    expect(setQueryData.mock.results[0]?.value).toEqual([drafts[1]])
  })

  it('patches cached folder lists for folder realtime events', () => {
    const foldersKey = ['communications-folders', undefined]
    const folder = {
      folder_id: 'folder-1',
      account_id: null,
      name: 'Projects',
      description: null,
      color: null,
      sort_order: 1000,
      message_count: 3,
      created_at: '2026-06-15T10:00:00Z',
      updated_at: '2026-06-15T10:00:00Z'
    }
    const folderData = {
      pages: [
        {
          items: [folder],
          next_cursor: null,
          has_more: false
        }
      ],
      pageParams: [null]
    }
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(folderData) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([[foldersKey, folderData]]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: '54',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.folder.updated',
            payload: {
              ...folder,
              name: 'Clients',
              message_count: 4,
              updated_at: '2026-06-15T10:01:00Z'
            }
          }
        })
      },
      queryClient
    )

    const patchedUpdate = setQueryData.mock.results[0]?.value
    expect(patchedUpdate.pages[0].items[0]).toMatchObject({
      folder_id: 'folder-1',
      name: 'Clients',
      message_count: 4
    })

    handleRealtimeEvent(
      {
        id: '55',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.folder.deleted',
            payload: folder
          }
        })
      },
      queryClient
    )

    const patchedDelete = setQueryData.mock.results[1]?.value
    expect(patchedDelete.pages[0].items).toEqual([])
  })
})
