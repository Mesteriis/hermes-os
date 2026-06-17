import type { TelegramMediaItem, TelegramMediaSearchResponse } from '../types/telegram'

export type TelegramMediaReadiness = {
  label: string
  detail: string
  can_preview_locally: boolean
  can_request_download: boolean
}

export function telegramMediaReadiness(item: TelegramMediaItem): TelegramMediaReadiness {
  if (item.local_path) {
    return {
      label: 'Preview ready',
      detail: 'Downloaded local file is available',
      can_preview_locally: true,
      can_request_download: false,
    }
  }

  if (item.tdlib_file_id !== null) {
    return {
      label: item.download_state === 'downloading' ? 'Download in progress' : 'Download available',
      detail: item.provider_attachment_id
        ? `TDLib file ${item.tdlib_file_id} · ${item.provider_attachment_id}`
        : `TDLib file ${item.tdlib_file_id}`,
      can_preview_locally: false,
      can_request_download: item.download_state !== 'downloading',
    }
  }

  return {
    label: 'Metadata only',
    detail: item.provider_attachment_id ?? 'No TDLib download handle projected',
    can_preview_locally: false,
    can_request_download: false,
  }
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
