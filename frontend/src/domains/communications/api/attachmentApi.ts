import {
  inspectAttachmentArchiveConnect,
  previewAttachmentConnect,
  searchAttachmentsConnect,
  translateAttachmentConnect
} from './connectCommunications'
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
  return searchAttachmentsConnect(request)
}

export async function inspectAttachmentArchive(
  attachmentId: string
): Promise<AttachmentArchiveInspectionResponse> {
  return inspectAttachmentArchiveConnect(attachmentId)
}

export async function previewAttachment(
  attachmentId: string
): Promise<AttachmentPreviewResponse> {
  return previewAttachmentConnect(attachmentId)
}

export async function translateAttachment(
  attachmentId: string,
  request: AttachmentTranslationRequest
): Promise<AttachmentTranslationResponse> {
  return translateAttachmentConnect(attachmentId, request.target_language, request.source_text)
}
