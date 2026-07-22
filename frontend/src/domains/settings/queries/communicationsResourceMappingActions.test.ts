import { describe, expect, it, vi } from 'vitest'
import { saveProviderResourceMappingAction } from './communicationsResourceMappingActions'
import type {
  MailProviderResource,
  MailProviderResourceMappingUpdate
} from '../../../shared/mailSync/providerResources'

describe('communications resource mapping actions', () => {
  it('does not update a read-only provider resource', async () => {
    const dependencies = dependenciesFor()

    await saveProviderResourceMappingAction('account-1', resource(false), update(), dependencies)

    expect(dependencies.updateProviderResourceMapping).not.toHaveBeenCalled()
  })

  it('persists writable mapping updates', async () => {
    const dependencies = dependenciesFor()

    await saveProviderResourceMappingAction('account-1', resource(true), update(), dependencies)

    expect(dependencies.updateProviderResourceMapping).toHaveBeenCalledWith({
      accountId: 'account-1',
      mappingId: 'mapping-1',
      update: update()
    })
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('Mail provider mapping saved')
  })
})

function dependenciesFor() {
  return {
    clearMessages: vi.fn(),
    setActionMessage: vi.fn(),
    setError: vi.fn(),
    updateProviderResourceMapping: vi.fn().mockResolvedValue({})
  }
}

function update(): MailProviderResourceMappingUpdate {
  return { semantic_role: 'inbox', local_folder_id: 'folder-1' }
}

function resource(writable: boolean): MailProviderResource {
  return {
    mapping_id: 'mapping-1',
    account_id: 'account-1',
    resource_kind: 'folder',
    provider_resource_id: 'INBOX',
    display_name: 'Inbox',
    semantic_role: 'inbox',
    local_folder_id: 'folder-1',
    selectable: true,
    writable,
    mapping_source: 'discovered',
    capabilities: {},
    observed_at: '2026-07-21T00:00:00Z',
    created_at: '2026-07-21T00:00:00Z',
    updated_at: '2026-07-21T00:00:00Z'
  }
}
