import { describe, expect, it, vi } from 'vitest'
import {
  invalidateTelegramMediaUploadState,
  primeTelegramUploadCommandQueues,
  telegramMediaTypeForFile,
} from './useTelegramMediaUploadWorkflow'
import { TELEGRAM_RUNTIME_COMMANDS_PAGE_SIZE } from '../../integrations/telegram/queries/telegramRuntimePanelActions'

describe('telegramMediaTypeForFile', () => {
  it('maps common desktop files to supported Telegram upload kinds', () => {
    expect(telegramMediaTypeForFile({ name: 'photo.jpg', type: 'image/jpeg' } as File)).toBe('photo')
    expect(telegramMediaTypeForFile({ name: 'clip.mp4', type: 'video/mp4' } as File)).toBe('video')
    expect(telegramMediaTypeForFile({ name: 'voice.ogg', type: 'audio/ogg' } as File)).toBe('audio')
    expect(telegramMediaTypeForFile({ name: 'fun.gif', type: 'image/gif' } as File)).toBe('animation')
    expect(telegramMediaTypeForFile({ name: 'archive.zip', type: 'application/zip' } as File)).toBe('document')
  })

  it('primes current command queues with a synthetic send_media row after upload success', () => {
    const commandsKey = ['integrations', 'telegram', 'commands', 'account-1', TELEGRAM_RUNTIME_COMMANDS_PAGE_SIZE]
    const commands: Array<Record<string, unknown>> = []
    const setQueryData = vi.fn()
    const queryClient = {
      getQueriesData: vi.fn().mockReturnValue([[commandsKey, commands]]),
      setQueryData,
    }

    primeTelegramUploadCommandQueues(
      queryClient,
      {
        command_id: 'cmd-upload-1',
        account_id: 'account-1',
        provider_chat_id: 'chat-1',
        attachment_id: 'att-1',
        blob_id: 'blob-1',
        media_type: 'document',
        status: 'queued',
        reconciliation_status: 'not_observed',
      },
      'upload-note.txt',
      'hello'
    )

    expect(setQueryData).toHaveBeenCalledOnce()
    expect(setQueryData.mock.calls[0][0]).toEqual(commandsKey)
    expect(setQueryData.mock.calls[0][1][0]).toMatchObject({
      command_id: 'cmd-upload-1',
      account_id: 'account-1',
      command_kind: 'send_media',
      provider_chat_id: 'chat-1',
      status: 'queued',
      reconciliation_status: 'not_observed',
    })
    expect(setQueryData.mock.calls[0][1][0].payload).toMatchObject({
      attachment_id: 'att-1',
      blob_id: 'blob-1',
      filename: 'upload-note.txt',
      caption: 'hello',
    })
  })

  it('refreshes the canonical Telegram conversation after an upload command is queued', () => {
    const invalidateQueries = vi.fn()

    invalidateTelegramMediaUploadState({ invalidateQueries } as never, 'account-1')

    expect(invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'telegram'],
    })
    expect(invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'telegram', 'commands', 'account-1'],
    })
  })
})
