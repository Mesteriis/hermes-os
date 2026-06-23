import {
  createCommunicationSavedSearchConnect,
  deleteCommunicationSavedSearchConnect,
  fetchCommunicationSavedSearchesConnect,
  updateCommunicationSavedSearchConnect
} from './connectCommunications'
import type {
  SavedSearchDeleteResponse,
  SavedSearchInput,
  SavedSearchListResponse,
  SavedSearchUpdate,
  CommunicationSavedSearch
} from '../types/savedSearches'

export async function fetchSavedSearches(
  smartFolder?: boolean,
  accountId?: string,
  limit = 500,
  cursor?: string | null
): Promise<SavedSearchListResponse> {
  return fetchCommunicationSavedSearchesConnect(smartFolder, accountId, limit, cursor ?? undefined)
}

export async function createSavedSearch(request: SavedSearchInput): Promise<CommunicationSavedSearch> {
  return createCommunicationSavedSearchConnect(request)
}

export async function updateSavedSearch(
  savedSearchId: string,
  request: SavedSearchUpdate
): Promise<CommunicationSavedSearch> {
  return updateCommunicationSavedSearchConnect(savedSearchId, request)
}

export async function deleteSavedSearch(savedSearchId: string): Promise<SavedSearchDeleteResponse> {
  return deleteCommunicationSavedSearchConnect(savedSearchId)
}
