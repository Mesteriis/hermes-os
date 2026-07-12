import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../platform/api/ApiClient'
import {
  fetchMailProviderResources,
  updateMailProviderResourceMapping,
} from './providerResources'

describe('mail provider resource mappings API', () => {
  beforeEach(() => ApiClient.init('http://127.0.0.1:8080', 'test-secret'))
  afterEach(() => {
    ApiClient.resetForTests()
    vi.unstubAllGlobals()
  })

  it('lists account-scoped resources and persists only manual mapping fields', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({
        items: [{
          mapping_id: 'mapping-1',
          account_id: 'account-1',
          resource_kind: 'folder',
          provider_resource_id: 'Sent Messages',
          display_name: 'Sent Messages',
          semantic_role: 'sent',
          local_folder_id: null,
          selectable: true,
          writable: true,
          mapping_source: 'discovered',
          capabilities: { imap_special_use: ['sent'] },
          observed_at: '2026-07-11T10:00:00Z',
          created_at: '2026-07-11T10:00:00Z',
          updated_at: '2026-07-11T10:00:00Z',
        }],
      }), { status: 200, headers: { 'Content-Type': 'application/json' } }))
      .mockResolvedValueOnce(new Response(JSON.stringify({
        mapping_id: 'mapping-1',
        account_id: 'account-1',
        resource_kind: 'folder',
        provider_resource_id: 'Sent Messages',
        display_name: 'Sent Messages',
        semantic_role: 'sent',
        local_folder_id: null,
        selectable: true,
        writable: true,
        mapping_source: 'manual',
        capabilities: {},
        observed_at: '2026-07-11T10:00:00Z',
        created_at: '2026-07-11T10:00:00Z',
        updated_at: '2026-07-11T10:01:00Z',
      }), { status: 200, headers: { 'Content-Type': 'application/json' } }))
    vi.stubGlobal('fetch', fetchMock)

    const resources = await fetchMailProviderResources('account-1')
    const updated = await updateMailProviderResourceMapping('account-1', 'mapping-1', {
      semantic_role: 'sent',
      local_folder_id: null,
    })

    expect(resources.items[0]?.provider_resource_id).toBe('Sent Messages')
    expect(updated.mapping_source).toBe('manual')
    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      'http://127.0.0.1:8080/api/v1/integrations/mail/accounts/account-1/provider-resources'
    )
    expect(fetchMock.mock.calls[1]?.[0]).toBe(
      'http://127.0.0.1:8080/api/v1/integrations/mail/accounts/account-1/provider-resources/mapping-1'
    )
    expect(JSON.parse(fetchMock.mock.calls[1]?.[1]?.body)).toEqual({
      semantic_role: 'sent',
      local_folder_id: null,
    })
  })
})
