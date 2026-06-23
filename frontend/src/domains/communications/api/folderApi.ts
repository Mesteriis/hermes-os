import {
  copyMessageToFolderConnect,
  createCommunicationFolderConnect,
  deleteCommunicationFolderConnect,
  fetchCommunicationFoldersConnect,
  fetchFolderMessagesConnect,
  moveMessageToFolderConnect,
  updateCommunicationFolderConnect
} from './connectCommunications'
import type {
  FolderDeleteResponse,
  FolderMessageActionResponse,
  FolderMessageListResponse,
  CommunicationFolder,
  CommunicationFolderInput,
  CommunicationFolderListResponse,
  CommunicationFolderUpdate
} from '../types/folders'

export async function fetchCommunicationFolders(
  accountId?: string,
  limit = 500,
  cursor?: string | null
): Promise<CommunicationFolderListResponse> {
  return fetchCommunicationFoldersConnect(accountId, limit, cursor ?? undefined)
}

export async function createCommunicationFolder(request: CommunicationFolderInput): Promise<CommunicationFolder> {
  return createCommunicationFolderConnect(request)
}

export async function updateCommunicationFolder(
  folderId: string,
  request: CommunicationFolderUpdate
): Promise<CommunicationFolder> {
  return updateCommunicationFolderConnect(folderId, request)
}

export async function deleteCommunicationFolder(folderId: string): Promise<FolderDeleteResponse> {
  return deleteCommunicationFolderConnect(folderId)
}

export async function fetchFolderMessages(
  folderId: string,
  limit = 250,
  cursor?: string | null
): Promise<FolderMessageListResponse> {
  return fetchFolderMessagesConnect(folderId, limit, cursor ?? undefined)
}

export async function copyMessageToFolder(
  folderId: string,
  messageId: string
): Promise<FolderMessageActionResponse> {
  return copyMessageToFolderConnect(folderId, messageId)
}

export async function moveMessageToFolder(
  folderId: string,
  messageId: string
): Promise<FolderMessageActionResponse> {
  return moveMessageToFolderConnect(folderId, messageId)
}
