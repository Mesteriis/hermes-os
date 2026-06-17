import { describe, expect, it } from 'vitest'
import { telegramMediaReadiness, telegramMediaSearchSourceLabel } from './telegramMediaSearch'
import type { TelegramMediaItem } from '../types/telegram'

function mediaItem(overrides: Partial<TelegramMediaItem>): TelegramMediaItem {
  return {
    message_id: 'msg-1',
    provider_message_id: 'provider-msg-1',
    provider_chat_id: 'chat-1',
    file_name: 'invoice.pdf',
    kind: 'document',
    mime_type: 'application/pdf',
    size_bytes: 2048,
    occurred_at: '2026-06-17T10:00:00Z',
    download_state: 'remote',
    tdlib_file_id: null,
    provider_attachment_id: null,
    local_path: null,
    ...overrides,
  }
}

describe('telegram media search projection', () => {
  it('marks downloaded media as locally previewable', () => {
    expect(
      telegramMediaReadiness(
        mediaItem({
          download_state: 'downloaded',
          local_path: '/tmp/hermes/invoice.pdf',
        })
      )
    ).toEqual({
      label: 'Preview ready',
      detail: 'Downloaded local file is available',
      can_preview_locally: true,
      can_request_download: false,
    })
  })

  it('marks TDLib-backed remote media as downloadable', () => {
    expect(
      telegramMediaReadiness(
        mediaItem({
          tdlib_file_id: 42,
          provider_attachment_id: 'attachment-42',
        })
      )
    ).toEqual({
      label: 'Download available',
      detail: 'TDLib file 42 · attachment-42',
      can_preview_locally: false,
      can_request_download: true,
    })
  })

  it('keeps metadata-only media separate from downloadable media', () => {
    expect(
      telegramMediaReadiness(
        mediaItem({
          provider_attachment_id: 'metadata-only',
        })
      )
    ).toMatchObject({
      label: 'Metadata only',
      detail: 'metadata-only',
      can_preview_locally: false,
      can_request_download: false,
    })
  })

  it('labels provider-refreshed media search and projection fallback states', () => {
    expect(telegramMediaSearchSourceLabel({
      query: 'invoice',
      source: 'provider_refresh',
      provider_search_attempted: true,
      provider_search_error: null,
      items: []
    })).toBe('Media search refreshed from provider before projection filtering.')
    expect(telegramMediaSearchSourceLabel({
      query: 'invoice',
      source: 'provider_refresh',
      provider_search_attempted: true,
      provider_search_error: 'TDLib unavailable',
      items: []
    })).toBe('Media provider refresh failed; showing local projection results.')
    expect(telegramMediaSearchSourceLabel({
      query: null,
      source: 'projection',
      provider_search_attempted: false,
      provider_search_error: null,
      items: []
    })).toBe('Media search used local projection metadata.')
  })
})
