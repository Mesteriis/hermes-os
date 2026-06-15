import { QueryClient } from '@tanstack/vue-query'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fetchMailMessage, fetchMailMessages, fetchThreadMessages } from '../api/communications'
import type { MailMessageDetailResponse, MailMessagesResponse, ThreadMessagesResponse } from '../types/communications'
import type { AttachmentSearchResult } from '../types/attachments'
import type { MailSavedSearch } from '../types/savedSearches'
import {
  mailListQueryKey,
  mailMessageQueryKey,
  prefetchMailMessageForAttachmentResult,
  prefetchMailListForSavedSearch,
  prefetchMailMessage,
  prefetchThreadMessages,
  threadMessagesQueryKey
} from './mailPrefetch'

vi.mock('../api/communications', () => ({
  fetchMailMessage: vi.fn(),
  fetchMailMessages: vi.fn(),
  fetchThreadMessages: vi.fn()
}))

const fetchMailMessageMock = vi.mocked(fetchMailMessage)
const fetchMailMessagesMock = vi.mocked(fetchMailMessages)
const fetchThreadMessagesMock = vi.mocked(fetchThreadMessages)

function messageDetail(messageId: string): MailMessageDetailResponse {
  return {
    message: {
      message_id: messageId,
      raw_record_id: 'raw-1',
      account_id: 'account-1',
      provider_record_id: 'provider-1',
      subject: 'Quarterly update',
      sender: 'sender@example.com',
      recipients: ['recipient@example.com'],
      body_text: 'Full body',
      body_html: null,
      occurred_at: '2026-06-14T10:00:00Z',
      projected_at: '2026-06-14T10:01:00Z',
      channel_kind: 'email',
      conversation_id: 'thread-1',
      sender_display_name: 'Sender',
      delivery_state: 'delivered',
      workflow_state: 'new',
      importance_score: null,
      ai_category: null,
      ai_summary: null,
      ai_summary_generated_at: null,
      message_metadata: {},
      local_state: 'active',
      local_state_changed_at: null,
      local_state_reason: null
    },
    attachments: []
  }
}

function queryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false
      }
    }
  })
}

function threadMessages(): ThreadMessagesResponse {
  return {
    items: []
  }
}

function mailMessages(): MailMessagesResponse {
  return {
    items: [],
    next_cursor: null,
    has_more: false
  }
}

function savedSearch(overrides: Partial<MailSavedSearch> = {}): MailSavedSearch {
  return {
    saved_search_id: 'search-1',
    name: 'Needs reply',
    description: null,
    account_id: 'account-1',
    query: 'quarterly',
    workflow_state: 'needs_action',
    local_state: 'active',
    channel_kind: 'email',
    is_smart_folder: false,
    sort_order: 0,
    message_count: 2,
    created_at: '2026-06-15T10:00:00Z',
    updated_at: '2026-06-15T10:00:00Z',
    ...overrides
  }
}

function attachmentSearchResult(overrides: Partial<AttachmentSearchResult> = {}): AttachmentSearchResult {
  return {
    attachment_id: 'attachment-1',
    message_id: 'msg-attachment-1',
    raw_record_id: 'raw-1',
    account_id: 'account-1',
    message_subject: 'Quarterly report',
    sender: 'sender@example.com',
    occurred_at: '2026-06-14T10:00:00Z',
    blob_id: 'blob-1',
    provider_attachment_id: 'provider-attachment-1',
    filename: 'report.pdf',
    content_type: 'application/pdf',
    size_bytes: 1024,
    sha256: 'hash-1',
    disposition: 'attachment',
    scan_status: 'not_scanned',
    scan_engine: null,
    scan_checked_at: null,
    scan_summary: null,
    storage_kind: 'local_blob',
    storage_path: 'mail/blob-1',
    created_at: '2026-06-14T10:00:00Z',
    updated_at: '2026-06-14T10:00:00Z',
    ...overrides
  }
}

describe('mail prefetch query helpers', () => {
  beforeEach(() => {
    fetchMailMessageMock.mockReset()
    fetchMailMessagesMock.mockReset()
    fetchThreadMessagesMock.mockReset()
  })

  it('prefetches message detail into the TanStack Query cache', async () => {
    const client = queryClient()
    const detail = messageDetail('msg-1')
    fetchMailMessageMock.mockResolvedValueOnce(detail)

    await prefetchMailMessage(client, ' msg-1 ')

    expect(fetchMailMessageMock).toHaveBeenCalledWith('msg-1')
    expect(client.getQueryData(mailMessageQueryKey('msg-1'))).toEqual(detail)
  })

  it('ignores blank message ids', async () => {
    const client = queryClient()

    await prefetchMailMessage(client, '  ')

    expect(fetchMailMessageMock).not.toHaveBeenCalled()
  })

  it('prefetches thread messages into the shared TanStack Query cache', async () => {
    const client = queryClient()
    const response = threadMessages()
    fetchThreadMessagesMock.mockResolvedValueOnce(response)

    await prefetchThreadMessages(client, ' account-1 ', ' Quarterly update ')

    expect(fetchThreadMessagesMock).toHaveBeenCalledWith('account-1', 'Quarterly update', 100)
    expect(client.getQueryData(threadMessagesQueryKey('account-1', 'Quarterly update'))).toEqual(response)
  })

  it('ignores blank thread prefetch inputs', async () => {
    const client = queryClient()

    await prefetchThreadMessages(client, 'account-1', '  ')

    expect(fetchThreadMessagesMock).not.toHaveBeenCalled()
  })

  it('prefetches the first mail list page for a saved search', async () => {
    const client = queryClient()
    const response = mailMessages()
    fetchMailMessagesMock.mockResolvedValueOnce(response)

    await prefetchMailListForSavedSearch(client, savedSearch(), 'fallback-account')

    expect(fetchMailMessagesMock).toHaveBeenCalledWith(
      'account-1',
      'needs_action',
      'email',
      'quarterly',
      'active',
      250,
      null
    )
    expect(client.getQueryData(mailListQueryKey('account-1', 'needs_action', 'email', 'quarterly', 'active'))).toEqual(response)
  })

  it('prefetches the parent message for an attachment search result', async () => {
    const client = queryClient()
    const detail = messageDetail('msg-attachment-1')
    fetchMailMessageMock.mockResolvedValueOnce(detail)

    await prefetchMailMessageForAttachmentResult(client, attachmentSearchResult())

    expect(fetchMailMessageMock).toHaveBeenCalledWith('msg-attachment-1')
    expect(client.getQueryData(mailMessageQueryKey('msg-attachment-1'))).toEqual(detail)
  })
})
