import { ApiClient } from '../../platform/api/ApiClient'

export type MailProviderResourceKind = 'folder' | 'label'

export type MailProviderSemanticRole =
  | 'inbox'
  | 'sent'
  | 'drafts'
  | 'archive'
  | 'trash'
  | 'junk'
  | 'all'
  | 'flagged'
  | 'important'
  | 'user'

export type MailProviderResource = {
  mapping_id: string
  account_id: string
  resource_kind: MailProviderResourceKind
  provider_resource_id: string
  display_name: string
  semantic_role: MailProviderSemanticRole | null
  local_folder_id: string | null
  selectable: boolean
  writable: boolean
  mapping_source: 'discovered' | 'manual'
  capabilities: Record<string, unknown>
  observed_at: string
  created_at: string
  updated_at: string
}

export type MailProviderResourceListResponse = {
  items: MailProviderResource[]
}

export type MailProviderResourceMappingUpdate = {
  semantic_role: MailProviderSemanticRole | null
  local_folder_id: string | null
}

export async function fetchMailProviderResources(
  accountId: string
): Promise<MailProviderResourceListResponse> {
  return ApiClient.instance.get<MailProviderResourceListResponse>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/provider-resources`,
    'Mail provider resource mapping request failed'
  )
}

export async function updateMailProviderResourceMapping(
  accountId: string,
  mappingId: string,
  update: MailProviderResourceMappingUpdate
): Promise<MailProviderResource> {
  return ApiClient.instance.put<MailProviderResource>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/provider-resources/${encodeURIComponent(mappingId)}`,
    update,
    'Mail provider resource mapping update failed'
  )
}
