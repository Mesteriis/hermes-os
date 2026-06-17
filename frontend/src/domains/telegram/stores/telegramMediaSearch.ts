import type { TelegramAttachmentHint, TelegramMediaItem, TelegramMediaSearchResponse } from '../types/telegram'

export type TelegramMediaReadiness = {
  label: string
  detail: string
  can_preview_locally: boolean
  can_request_download: boolean
  action_label: string
}

export function telegramMediaReadiness(item: TelegramMediaItem): TelegramMediaReadiness {
  return mediaReadiness({
    localPath: item.local_path,
    tdlibFileId: item.tdlib_file_id,
    providerAttachmentId: item.provider_attachment_id,
    downloadState: item.download_state,
    expectedSizeBytes: item.expected_size_bytes ?? null,
    downloadedSizeBytes: item.downloaded_size_bytes ?? null,
    isDownloadingActive: item.is_downloading_active ?? null,
    isDownloadingCompleted: item.is_downloading_completed ?? null,
    lastError: item.last_error ?? null,
  })
}

export function telegramAttachmentReadiness(attachment: TelegramAttachmentHint): TelegramMediaReadiness {
  return mediaReadiness(attachment)
}

export function telegramMediaSearchSourceLabel(response: TelegramMediaSearchResponse | null | undefined): string {
  if (!response) return ''
  if (response.provider_search_error) {
    return 'Media provider refresh failed; showing local projection results.'
  }
  if (response.provider_search_attempted && response.source === 'provider_refresh') {
    return 'Media search refreshed from provider before projection filtering.'
  }
  return 'Media search used local projection metadata.'
}

function mediaReadiness(item: {
  localPath: string | null
  tdlibFileId: number | null
  providerAttachmentId?: string | null
  downloadState: string
  expectedSizeBytes?: number | null
  downloadedSizeBytes?: number | null
  isDownloadingActive?: boolean | null
  isDownloadingCompleted?: boolean | null
  lastError?: string | null
}): TelegramMediaReadiness {
  if (item.localPath) {
    return {
      label: 'Preview ready',
      detail: 'Downloaded local file is available',
      can_preview_locally: true,
      can_request_download: false,
      action_label: 'Downloaded',
    }
  }

  if (item.downloadState === 'downloading') {
    return {
      label: 'Download in progress',
      detail: downloadProgressDetail(item),
      can_preview_locally: false,
      can_request_download: false,
      action_label: 'Downloading',
    }
  }

  if (item.downloadState === 'failed') {
    return {
      label: 'Download failed',
      detail: item.lastError?.trim() || tdlibDetail(item),
      can_preview_locally: false,
      can_request_download: item.tdlibFileId !== null,
      action_label: 'Retry download',
    }
  }

  if (item.tdlibFileId !== null) {
    return {
      label: 'Download available',
      detail: tdlibDetail(item),
      can_preview_locally: false,
      can_request_download: true,
      action_label: 'Download',
    }
  }

  return {
    label: 'Metadata only',
    detail: item.providerAttachmentId ?? 'No TDLib download handle projected',
    can_preview_locally: false,
    can_request_download: false,
    action_label: 'Unavailable',
  }
}

function tdlibDetail(item: { tdlibFileId: number | null; providerAttachmentId?: string | null }): string {
  if (item.tdlibFileId === null) {
    return item.providerAttachmentId ?? 'No TDLib download handle projected'
  }
  return item.providerAttachmentId
    ? `TDLib file ${item.tdlibFileId} · ${item.providerAttachmentId}`
    : `TDLib file ${item.tdlibFileId}`
}

function downloadProgressDetail(item: {
  expectedSizeBytes?: number | null
  downloadedSizeBytes?: number | null
  isDownloadingActive?: boolean | null
  isDownloadingCompleted?: boolean | null
}): string {
  const expected = item.expectedSizeBytes ?? null
  const downloaded = item.downloadedSizeBytes ?? null
  if (typeof expected === 'number' && expected > 0 && typeof downloaded === 'number' && downloaded >= 0) {
    const percent = Math.max(0, Math.min(100, Math.round((downloaded / expected) * 100)))
    return `${percent}% · ${downloaded}/${expected} bytes`
  }
  if (item.isDownloadingCompleted === true) return 'TDLib reported download completion'
  if (item.isDownloadingActive === true) return 'TDLib download is active'
  return 'Waiting for provider progress update'
}
