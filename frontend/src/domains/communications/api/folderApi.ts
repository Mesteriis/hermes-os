import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  FolderDeleteResponse,
  FolderMessageActionResponse,
  FolderMessageListResponse,
  MailFolder,
  MailFolderInput,
  MailFolderListResponse,
  MailFolderUpdate
} from '../types/folders'

export async function fetchMailFolders(
  accountId?: string,
  limit = 500,
  cursor?: string | null
): Promise<MailFolderListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  if (cursor?.trim()) params.set('cursor', cursor.trim())
  return ApiClient.instance.get<MailFolderListResponse>(
    `/api/v1/communications/folders?${params.toString()}`,
    'Mail folders request failed'
  )
}

export async function createMailFolder(request: MailFolderInput): Promise<MailFolder> {
  return ApiClient.instance.post<MailFolder>(
    '/api/v1/communications/folders',
    request,
    'Mail folder creation failed'
  )
}

export async function updateMailFolder(
  folderId: string,
  request: MailFolderUpdate
): Promise<MailFolder> {
  return ApiClient.instance.put<MailFolder>(
    `/api/v1/communications/folders/${encodeURIComponent(folderId)}`,
    request,
    'Mail folder update failed'
  )
}

export async function deleteMailFolder(folderId: string): Promise<FolderDeleteResponse> {
  return ApiClient.instance.delete<FolderDeleteResponse>(
    `/api/v1/communications/folders/${encodeURIComponent(folderId)}`,
    'Mail folder deletion failed'
  )
}

export async function fetchFolderMessages(
  folderId: string,
  limit = 250,
  cursor?: string | null
): Promise<FolderMessageListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (cursor?.trim()) params.set('cursor', cursor.trim())
  return ApiClient.instance.get<FolderMessageListResponse>(
    `/api/v1/communications/folders/${encodeURIComponent(folderId)}/messages?${params.toString()}`,
    'Folder messages request failed'
  )
}

export async function copyMessageToFolder(
  folderId: string,
  messageId: string
): Promise<FolderMessageActionResponse> {
  return ApiClient.instance.post<FolderMessageActionResponse>(
    `/api/v1/communications/folders/${encodeURIComponent(folderId)}/messages/${encodeURIComponent(
      messageId
    )}/copy`,
    {},
    'Copy message to folder failed'
  )
}

export async function moveMessageToFolder(
  folderId: string,
  messageId: string
): Promise<FolderMessageActionResponse> {
  return ApiClient.instance.post<FolderMessageActionResponse>(
    `/api/v1/communications/folders/${encodeURIComponent(folderId)}/messages/${encodeURIComponent(
      messageId
    )}/move`,
    {},
    'Move message to folder failed'
  )
}
