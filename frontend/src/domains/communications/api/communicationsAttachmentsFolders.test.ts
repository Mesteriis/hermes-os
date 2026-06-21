import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  copyMessageToFolder,
  createCommunicationFolder,
  fetchFolderMessages,
  fetchCommunicationFolders,
  inspectAttachmentArchive,
  moveMessageToFolder,
  previewAttachment,
  redirectMessage,
  searchAttachments,
  translateAttachment,
  translateThread
} from './communications'

describe('communications API attachment and folder helpers', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('searches attachment metadata with cursor filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [], next_cursor: null, has_more: false }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await searchAttachments({
      account_id: 'account-1',
      q: 'invoice pdf',
      content_type: 'pdf',
      scan_status: 'not_scanned',
      limit: 25,
      cursor: 'cursor:value'
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/attachments/search?')
    expect(url).toContain('account_id=account-1')
    expect(url).toContain('q=invoice+pdf')
    expect(url).toContain('content_type=pdf')
    expect(url).toContain('scan_status=not_scanned')
    expect(url).toContain('limit=25')
    expect(url).toContain('cursor=cursor%3Avalue')
  })

  it('posts attachment translation requests with provided extracted text', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          attachment_id: 'mail_attachment:1',
          translated: false,
          reason: 'translation runtime unavailable'
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    await translateAttachment('mail_attachment:1', {
      target_language: 'en',
      source_text: 'Hola equipo'
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/attachments/mail_attachment%3A1/translate')
    expect(init.method).toBe('POST')
    expect(JSON.parse(init.body as string)).toEqual({
      target_language: 'en',
      source_text: 'Hola equipo'
    })
  })

  it('fetches attachment archive inspection reports by attachment id', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          attachment_id: 'mail_attachment:1',
          report: {
            archive_kind: 'zip',
            entry_count: 1,
            total_uncompressed_bytes: 5,
            has_nested_archive: false,
            entries: []
          }
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const report = await inspectAttachmentArchive('mail_attachment:1')

    expect(report.report.archive_kind).toBe('zip')
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/attachments/mail_attachment%3A1/archive-inspection')
    expect(init.method).toBe('GET')
  })

  it('fetches safe attachment previews by attachment id', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          attachment_id: 'mail_attachment:1',
          preview_kind: 'text',
          text: 'First line',
          data_url: null,
          truncated: false
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const preview = await previewAttachment('mail_attachment:1')

    expect(preview.preview_kind).toBe('text')
    expect(preview.text).toBe('First line')
    expect(preview.data_url).toBeNull()
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/attachments/mail_attachment%3A1/preview')
    expect(init.method).toBe('GET')
  })

  it('manages custom folders and local folder message actions', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            folder_id: 'mail_folder:1',
            account_id: 'account-1',
            name: 'Clients',
            description: null,
            color: '#3b82f6',
            sort_order: 10,
            message_count: 3,
            created_at: '2026-06-14T00:00:00Z',
            updated_at: '2026-06-14T00:00:00Z'
          }],
          next_cursor: null,
          has_more: false
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ folder_id: 'mail_folder:1', name: 'Clients' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ operation: 'copy', folder_id: 'mail_folder:1', message_id: 'msg-1' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ operation: 'move', folder_id: 'mail_folder:1', message_id: 'msg-1' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [], next_cursor: null, has_more: false }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const folders = await fetchCommunicationFolders('account-1', 50)
    await createCommunicationFolder({
      name: 'Clients',
      account_id: 'account-1',
      color: '#3b82f6',
      sort_order: 10
    })
    await copyMessageToFolder('mail_folder:1', 'msg-1')
    await moveMessageToFolder('mail_folder:1', 'msg-1')
    await fetchFolderMessages('mail_folder:1', 25, 'cursor:value')

    expect(fetchMock).toHaveBeenCalledTimes(5)
    expect(folders.items[0].message_count).toBe(3)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/communications/folders?')
    expect(fetchMock.mock.calls[0][0]).toContain('account_id=account-1')
    expect(fetchMock.mock.calls[1][1].method).toBe('POST')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/communications/folders')
    expect(JSON.parse(fetchMock.mock.calls[1][1].body as string)).toEqual({
      name: 'Clients',
      account_id: 'account-1',
      color: '#3b82f6',
      sort_order: 10
    })
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/communications/folders/mail_folder%3A1/messages/msg-1/copy')
    expect(fetchMock.mock.calls[2][1].method).toBe('POST')
    expect(fetchMock.mock.calls[3][0]).toContain('/api/v1/communications/folders/mail_folder%3A1/messages/msg-1/move')
    expect(fetchMock.mock.calls[4][0]).toContain('/api/v1/communications/folders/mail_folder%3A1/messages?')
    expect(fetchMock.mock.calls[4][0]).toContain('limit=25')
    expect(fetchMock.mock.calls[4][0]).toContain('cursor=cursor%3Avalue')
  })

  it('posts thread translation requests with account and subject filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [], target_language: 'en' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await translateThread('account-1', 'Thread Translation', 'en')

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/threads/translate?')
    expect(url).toContain('account_id=account-1')
    expect(url).toContain('subject=Thread+Translation')
    expect(init.method).toBe('POST')
    expect(JSON.parse(init.body as string)).toEqual({ target_language: 'en' })
  })

  it('posts redirect requests to the message redirect endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ outbox_id: 'outbox-1', status: 'queued' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await redirectMessage('msg-1', {
      to: ['redirect@example.com'],
      cc: ['copy@example.com'],
      confirmed_provider_write: true
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/messages/msg-1/redirect')
    expect(init.method).toBe('POST')
    expect(JSON.parse(init.body as string)).toEqual({
      to: ['redirect@example.com'],
      cc: ['copy@example.com'],
      confirmed_provider_write: true
    })
  })
})
