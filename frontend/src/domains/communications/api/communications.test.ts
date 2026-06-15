import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  bulkMessageAction,
  createSavedSearch,
  deleteSavedSearch,
  deleteRichTemplate,
  fetchMailMessages,
  fetchDrafts,
  fetchSubscriptions,
  fetchTopSenders,
  fetchRichTemplates,
  fetchOutboxItems,
  fetchThreadMessages,
  fetchThreads,
  previewRichTemplateMailMerge,
  renderRichTemplate,
  saveRichTemplate,
  fetchSavedSearches,
  undoOutboxItem,
  updateSavedSearch
} from './communications'

describe('communications API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('passes cursor pagination parameters to the mail messages endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [], next_cursor: 'next-cursor', has_more: true }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchMailMessages(
      'account-1',
      'new',
      'email',
      'quarterly update',
      'active',
      50,
      'cursor:value'
    )

    expect(response.next_cursor).toBe('next-cursor')
    expect(response.has_more).toBe(true)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/messages?')
    expect(url).toContain('account_id=account-1')
    expect(url).toContain('workflow_state=new')
    expect(url).toContain('channel_kind=email')
    expect(url).toContain('q=quarterly+update')
    expect(url).toContain('local_state=active')
    expect(url).toContain('limit=50')
    expect(url).toContain('cursor=cursor%3Avalue')
  })

  it('fetches outbox items with account, status and cursor filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [], next_cursor: 'next-cursor', has_more: true }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchOutboxItems('account-1', 'scheduled', 25, 'cursor:value')

    expect(response.next_cursor).toBe('next-cursor')
    expect(response.has_more).toBe(true)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/outbox?')
    expect(url).toContain('account_id=account-1')
    expect(url).toContain('status=scheduled')
    expect(url).toContain('limit=25')
    expect(url).toContain('cursor=cursor%3Avalue')
  })

  it('fetches drafts with account, status and cursor filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [], next_cursor: 'next-draft-cursor', has_more: true }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchDrafts('account-1', 'draft', 25, 'cursor:value')

    expect(response.next_cursor).toBe('next-draft-cursor')
    expect(response.has_more).toBe(true)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/drafts?')
    expect(url).toContain('account_id=account-1')
    expect(url).toContain('status=draft')
    expect(url).toContain('limit=25')
    expect(url).toContain('cursor=cursor%3Avalue')
  })

  it('fetches subscriptions with account and cursor filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [], next_cursor: 'next-subscription-cursor', has_more: true }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchSubscriptions('account-1', 25, 'cursor:value')

    expect(response.next_cursor).toBe('next-subscription-cursor')
    expect(response.has_more).toBe(true)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/subscriptions?')
    expect(url).toContain('account_id=account-1')
    expect(url).toContain('limit=25')
    expect(url).toContain('cursor=cursor%3Avalue')
  })

  it('fetches top senders with account and cursor filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [], next_cursor: 'next-sender-cursor', has_more: true }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchTopSenders('account-1', 25, 'cursor:value')

    expect(response.next_cursor).toBe('next-sender-cursor')
    expect(response.has_more).toBe(true)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/analytics/senders?')
    expect(url).toContain('account_id=account-1')
    expect(url).toContain('limit=25')
    expect(url).toContain('cursor=cursor%3Avalue')
  })

  it('fetches threads with account and cursor filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [], next_cursor: 'next-thread-cursor', has_more: true }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchThreads('account-1', 25, 'cursor:value')

    expect(response.next_cursor).toBe('next-thread-cursor')
    expect(response.has_more).toBe(true)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/threads?')
    expect(url).toContain('account_id=account-1')
    expect(url).toContain('limit=25')
    expect(url).toContain('cursor=cursor%3Avalue')
  })

  it('preserves provider record ids in thread message responses for inline replies', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        items: [{
          message_id: 'msg-1',
          provider_record_id: 'provider-msg-1',
          account_id: 'account-1',
          subject: 'Quarterly update',
          sender: 'Ada <ada@example.com>',
          sender_display_name: 'Ada',
          body_text: 'Thread body',
          occurred_at: null,
          projected_at: '2026-06-15T10:00:00Z',
          workflow_state: 'new',
          importance_score: null,
          ai_category: null,
          ai_summary: null,
          delivery_state: 'received',
          attachment_count: 0
        }]
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchThreadMessages('account-1', 'Quarterly update', 25)

    expect(response.items[0].provider_record_id).toBe('provider-msg-1')
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/threads/messages?')
    expect(url).toContain('account_id=account-1')
    expect(url).toContain('subject=Quarterly+update')
    expect(url).toContain('limit=25')
  })

  it('undoes an outbox item through the protected communications API', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ outbox_id: 'outbox-1', status: 'canceled' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await undoOutboxItem('outbox-1')

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/outbox/outbox-1/undo')
    expect(init.method).toBe('POST')
  })

  it('saves, lists, and renders durable rich templates through the communications API', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          saved: true,
          template: {
            template_id: 'tpl-1',
            name: 'Intro',
            placeholder_variables: ['name'],
            undeclared_variables: [],
            unused_variables: [],
            malformed_placeholders: []
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          saved: true,
          template: {
            template_id: 'tpl-1',
            name: 'Intro',
            placeholder_variables: ['name'],
            undeclared_variables: [],
            unused_variables: [],
            malformed_placeholders: []
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          templates: [{
            template_id: 'tpl-1',
            name: 'Intro',
            placeholder_variables: ['name'],
            undeclared_variables: [],
            unused_variables: [],
            malformed_placeholders: []
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          template_id: 'tpl-1',
          rendered: {
            subject: 'Hello Alex',
            body: '<p>Welcome</p>',
            missing_variables: [],
            unresolved_variables: [],
            malformed_placeholders: []
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          template_id: 'tpl-1',
          row_count: 2,
          ready_count: 1,
          blocked_count: 1,
          items: [
            {
              row_id: 'r1',
              ready: true,
              rendered: {
                subject: 'Hello Alex',
                body: '<p>Welcome</p>',
                missing_variables: [],
                unresolved_variables: [],
                malformed_placeholders: []
              }
            },
            {
              row_id: 'r2',
              ready: false,
              rendered: {
                subject: 'Hello {{ name }}',
                body: '<p>{{ name }}</p>',
                missing_variables: ['name'],
                unresolved_variables: ['name'],
                malformed_placeholders: []
              }
            }
          ]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ template_id: 'tpl-1', deleted: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const saved = await saveRichTemplate({
      name: 'Intro',
      subject_template: 'Hello {{name}}',
      body_template: '<p>Welcome {{name}}</p>',
      variables: ['name'],
      language: null
    })
    await saveRichTemplate({
      template_id: 'tpl-1',
      name: 'Intro',
      subject_template: 'Updated {{name}}',
      body_template: '<p>Updated {{name}}</p>',
      variables: ['name'],
      language: null
    })
    const templates = await fetchRichTemplates()
    const rendered = await renderRichTemplate({
      template_id: 'tpl-1',
      variables: { name: 'Alex' }
    })
    const preview = await previewRichTemplateMailMerge({
      template_id: 'tpl-1',
      rows: [
        { row_id: 'r1', variables: { name: 'Alex' } },
        { row_id: 'r2', variables: {} }
      ]
    })
    await deleteRichTemplate('tpl-1')

    expect(saved.template.placeholder_variables).toEqual(['name'])
    expect(saved.template.undeclared_variables).toEqual([])
    expect(templates.templates[0].malformed_placeholders).toEqual([])
    expect(rendered.rendered.subject).toBe('Hello Alex')
    expect(rendered.rendered.missing_variables).toEqual([])
    expect(rendered.rendered.unresolved_variables).toEqual([])
    expect(rendered.rendered.malformed_placeholders).toEqual([])
    expect(preview.ready_count).toBe(1)
    expect(preview.blocked_count).toBe(1)
    expect(preview.items[1].rendered.missing_variables).toEqual(['name'])
    expect(fetchMock).toHaveBeenCalledTimes(6)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/communications/templates/rich')
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body as string)).toEqual({
      name: 'Intro',
      subject_template: 'Hello {{name}}',
      body_template: '<p>Welcome {{name}}</p>',
      variables: ['name'],
      language: null
    })
    expect(JSON.parse(fetchMock.mock.calls[1][1].body as string)).toEqual({
      template_id: 'tpl-1',
      name: 'Intro',
      subject_template: 'Updated {{name}}',
      body_template: '<p>Updated {{name}}</p>',
      variables: ['name'],
      language: null
    })
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/communications/templates/rich')
    const [url, init] = fetchMock.mock.calls[3]
    expect(url).toContain('/api/v1/communications/templates/rich/render')
    expect(init.method).toBe('POST')
    expect(JSON.parse(init.body as string)).toEqual({
      template_id: 'tpl-1',
      variables: { name: 'Alex' }
    })
    const [previewUrl, previewInit] = fetchMock.mock.calls[4]
    expect(previewUrl).toContain('/api/v1/communications/templates/rich/mail-merge-preview')
    expect(previewInit.method).toBe('POST')
    expect(JSON.parse(previewInit.body as string)).toEqual({
      template_id: 'tpl-1',
      rows: [
        { row_id: 'r1', variables: { name: 'Alex' } },
        { row_id: 'r2', variables: {} }
      ]
    })
    expect(fetchMock.mock.calls[5][0]).toContain('/api/v1/communications/templates/rich/tpl-1')
    expect(fetchMock.mock.calls[5][1].method).toBe('DELETE')
  })

  it('posts bounded message bulk actions to the communications API', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        action: 'trash',
        matched_count: 2,
        updated_count: 2,
        not_found: []
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await bulkMessageAction({
      action: 'trash',
      message_ids: ['msg-1', 'msg-2']
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/messages/bulk-actions')
    expect(init.method).toBe('POST')
    expect(JSON.parse(init.body as string)).toEqual({
      action: 'trash',
      message_ids: ['msg-1', 'msg-2']
    })
  })

  it('manages saved searches through the communications API', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [], next_cursor: 'next-saved-search-cursor', has_more: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ saved_search_id: 'mail_saved_search:1', name: 'Invoices' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ saved_search_id: 'mail_saved_search:1', name: 'Waiting invoices' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ deleted: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const savedSearches = await fetchSavedSearches(true, 'account-1', 25, 'cursor:value')
    await createSavedSearch({
      name: 'Invoices',
      query: 'invoice',
      workflow_state: 'needs_action',
      local_state: 'active',
      is_smart_folder: true
    })
    await updateSavedSearch('mail_saved_search:1', {
      name: 'Waiting invoices',
      workflow_state: 'waiting'
    })
    await deleteSavedSearch('mail_saved_search:1')

    expect(fetchMock).toHaveBeenCalledTimes(4)
    expect(savedSearches.next_cursor).toBe('next-saved-search-cursor')
    expect(savedSearches.has_more).toBe(true)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/communications/saved-searches?smart_folder=true')
    expect(fetchMock.mock.calls[0][0]).toContain('account_id=account-1')
    expect(fetchMock.mock.calls[0][0]).toContain('limit=25')
    expect(fetchMock.mock.calls[0][0]).toContain('cursor=cursor%3Avalue')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/communications/saved-searches')
    expect(fetchMock.mock.calls[1][1].method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[1][1].body as string)).toEqual({
      name: 'Invoices',
      query: 'invoice',
      workflow_state: 'needs_action',
      local_state: 'active',
      is_smart_folder: true
    })
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/communications/saved-searches/mail_saved_search%3A1')
    expect(fetchMock.mock.calls[2][1].method).toBe('PUT')
    expect(fetchMock.mock.calls[3][0]).toContain('/api/v1/communications/saved-searches/mail_saved_search%3A1')
    expect(fetchMock.mock.calls[3][1].method).toBe('DELETE')
  })

})
