import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../platform/api/ApiClient'
import {
  fetchMailContentEgressSettings,
  updateMailContentEgressSettings,
} from './syncApi'

describe('mail content egress settings API', () => {
  beforeEach(() => ApiClient.init('http://127.0.0.1:8080', 'test-secret'))
  afterEach(() => {
    ApiClient.resetForTests()
    vi.unstubAllGlobals()
  })

  it('uses an account-scoped endpoint and persists only the changed permission', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({
        body: false,
        attachments: false,
        extracted_text: false,
      }), { status: 200, headers: { 'Content-Type': 'application/json' } }))
      .mockResolvedValueOnce(new Response(JSON.stringify({
        body: true,
        attachments: false,
        extracted_text: false,
      }), { status: 200, headers: { 'Content-Type': 'application/json' } }))
    vi.stubGlobal('fetch', fetchMock)

    const settings = await fetchMailContentEgressSettings('account-1')
    const updated = await updateMailContentEgressSettings('account-1', { body: true })

    expect(settings).toEqual({ body: false, attachments: false, extracted_text: false })
    expect(updated.body).toBe(true)
    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      'http://127.0.0.1:8080/api/v1/integrations/mail/accounts/account-1/content-egress-settings'
    )
    expect(fetchMock.mock.calls[1]?.[0]).toBe(
      'http://127.0.0.1:8080/api/v1/integrations/mail/accounts/account-1/content-egress-settings'
    )
    expect(JSON.parse(fetchMock.mock.calls[1]?.[1]?.body)).toEqual({ body: true })
  })
})
