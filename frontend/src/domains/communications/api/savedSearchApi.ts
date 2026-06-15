import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  SavedSearchDeleteResponse,
  SavedSearchInput,
  SavedSearchListResponse,
  SavedSearchUpdate,
  MailSavedSearch
} from '../types/savedSearches'

export async function fetchSavedSearches(
  smartFolder?: boolean,
  accountId?: string,
  limit = 500,
  cursor?: string | null
): Promise<SavedSearchListResponse> {
  const params = new URLSearchParams()
  if (typeof smartFolder === 'boolean') params.set('smart_folder', String(smartFolder))
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  if (cursor?.trim()) params.set('cursor', cursor.trim())
  params.set('limit', String(Math.trunc(limit)))
  return ApiClient.instance.get<SavedSearchListResponse>(
    `/api/v1/communications/saved-searches?${params.toString()}`,
    'Saved searches request failed'
  )
}

export async function createSavedSearch(request: SavedSearchInput): Promise<MailSavedSearch> {
  return ApiClient.instance.post<MailSavedSearch>(
    '/api/v1/communications/saved-searches',
    request,
    'Saved search creation failed'
  )
}

export async function updateSavedSearch(
  savedSearchId: string,
  request: SavedSearchUpdate
): Promise<MailSavedSearch> {
  return ApiClient.instance.put<MailSavedSearch>(
    `/api/v1/communications/saved-searches/${encodeURIComponent(savedSearchId)}`,
    request,
    'Saved search update failed'
  )
}

export async function deleteSavedSearch(savedSearchId: string): Promise<SavedSearchDeleteResponse> {
  return ApiClient.instance.delete<SavedSearchDeleteResponse>(
    `/api/v1/communications/saved-searches/${encodeURIComponent(savedSearchId)}`,
    'Saved search deletion failed'
  )
}
