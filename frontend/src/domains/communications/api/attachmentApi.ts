import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  AttachmentArchiveInspectionResponse,
  AttachmentPreviewResponse,
  AttachmentSearchRequest,
  AttachmentSearchResponse,
  AttachmentTranslationRequest,
  AttachmentTranslationResponse
} from '../types/attachments'

export async function searchAttachments(
  request: AttachmentSearchRequest = {}
): Promise<AttachmentSearchResponse> {
  const params = new URLSearchParams()
  if (request.account_id?.trim()) params.set('account_id', request.account_id.trim())
  if (request.q?.trim()) params.set('q', request.q.trim())
  if (request.content_type?.trim()) params.set('content_type', request.content_type.trim())
  if (request.scan_status?.trim()) params.set('scan_status', request.scan_status.trim())
  if (request.cursor?.trim()) params.set('cursor', request.cursor.trim())
  params.set('limit', String(Math.trunc(request.limit ?? 100)))
  return ApiClient.instance.get<AttachmentSearchResponse>(
    `/api/v1/communications/attachments/search?${params.toString()}`,
    'Attachment search request failed'
  )
}

export async function inspectAttachmentArchive(
  attachmentId: string
): Promise<AttachmentArchiveInspectionResponse> {
  return ApiClient.instance.get<AttachmentArchiveInspectionResponse>(
    `/api/v1/communications/attachments/${encodeURIComponent(attachmentId)}/archive-inspection`,
    'Attachment archive inspection failed'
  )
}

export async function previewAttachment(
  attachmentId: string
): Promise<AttachmentPreviewResponse> {
  return ApiClient.instance.get<AttachmentPreviewResponse>(
    `/api/v1/communications/attachments/${encodeURIComponent(attachmentId)}/preview`,
    'Attachment preview failed'
  )
}

export async function translateAttachment(
  attachmentId: string,
  request: AttachmentTranslationRequest
): Promise<AttachmentTranslationResponse> {
  return ApiClient.instance.post<AttachmentTranslationResponse>(
    `/api/v1/communications/attachments/${encodeURIComponent(attachmentId)}/translate`,
    request,
    'Attachment translation failed'
  )
}
