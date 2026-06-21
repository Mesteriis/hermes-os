import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { z } from 'zod'
import { importCommunicationAttachment } from '../../domains/communications/api/attachmentImportApi'
import {
  uploadTelegramMedia,
  type TelegramMediaUploadKind,
} from '../../integrations/telegram/api/telegramMediaUpload'
import { patchTelegramCommandList } from '../../integrations/telegram/queries/realtimeTelegramCommandPatches'
import { telegramQueryKeys } from '../../integrations/telegram/queries/useTelegramQuery'
import type { TelegramProviderWriteCommand } from '../../integrations/telegram/types/telegram'

export type TelegramMediaUploadInput = {
  accountId: string
  providerChatId: string
  file: File
  caption?: string
}

const uploadInputSchema = z.object({
  accountId: z.string().trim().min(1),
  providerChatId: z.string().trim().min(1),
  file: z.custom<File>(
    (value) =>
      typeof value === 'object' &&
      value !== null &&
      typeof (value as File).arrayBuffer === 'function' &&
      typeof (value as File).size === 'number' &&
      (value as File).size > 0,
    'file is required'
  ),
  caption: z.string().trim().min(1).optional(),
})

export function useTelegramMediaUploadMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: uploadTelegramMediaFile,
    onSuccess: (result, variables) => {
      primeTelegramUploadCommandQueues(queryClient, result, variables.file.name || undefined, variables.caption)
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
      queryClient.invalidateQueries({ queryKey: ['integrations', 'telegram', 'commands', result.account_id] })
      queryClient.invalidateQueries({ queryKey: ['integrations', 'telegram', 'commands'] })
    },
  })
}

export function primeTelegramUploadCommandQueues(
  queryClient: {
    getQueriesData: <TData>(filters: { queryKey: readonly unknown[] }) => Array<
      [readonly unknown[], TData | undefined]
    >
    setQueryData: <TData>(queryKey: readonly unknown[], updater: TData) => unknown
  },
  result: Awaited<ReturnType<typeof uploadTelegramMedia>>,
  filename?: string,
  caption?: string
): void {
  for (const [queryKey, data] of queryClient.getQueriesData<TelegramProviderWriteCommand[]>({
    queryKey: ['integrations', 'telegram', 'commands'],
  })) {
    const updated = patchTelegramCommandList(
      queryKey,
      data,
      'telegram.media.upload.started',
      {
        command_id: result.command_id,
        account_id: result.account_id,
        provider_chat_id: result.provider_chat_id,
        command_kind: 'send_media',
        idempotency_key: result.command_id,
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'confirmed',
        status: result.status,
        retry_count: 0,
        max_retries: 3,
        reconciliation_status: result.reconciliation_status,
        payload: {
          attachment_id: result.attachment_id,
          blob_id: result.blob_id,
          media_type: result.media_type,
          filename: filename?.trim() || undefined,
          caption: caption?.trim() || undefined,
        },
        target_ref: {
          provider_chat_id: result.provider_chat_id,
          attachment_id: result.attachment_id,
          blob_id: result.blob_id,
        },
      }
    )
    if (updated !== data) {
      queryClient.setQueryData(queryKey, updated)
    }
  }
}

export async function uploadTelegramMediaFile(input: TelegramMediaUploadInput) {
  const parsed = uploadInputSchema.parse(input)
  const contentBase64 = await fileToBase64(parsed.file)
  const imported = await importCommunicationAttachment({
    account_id: parsed.accountId.trim(),
    channel_kind: 'telegram',
    filename: parsed.file.name || undefined,
    content_type: parsed.file.type || 'application/octet-stream',
    content_base64: contentBase64,
    source_kind: 'telegram_composer',
    metadata: {
      composer: 'telegram',
      last_modified: parsed.file.lastModified || undefined,
    },
  })

  return uploadTelegramMedia({
    account_id: parsed.accountId.trim(),
    provider_chat_id: parsed.providerChatId.trim(),
    attachment_id: imported.attachment_id,
    blob_id: imported.blob_id,
    media_type: telegramMediaTypeForFile(parsed.file),
    caption: parsed.caption?.trim(),
    filename: parsed.file.name || undefined,
  })
}

export function telegramMediaTypeForFile(file: Pick<File, 'name' | 'type'>): TelegramMediaUploadKind {
  const type = file.type.toLowerCase()
  const name = file.name.toLowerCase()
  if (type === 'image/gif' || name.endsWith('.gif')) return 'animation'
  if (type.startsWith('image/')) return 'photo'
  if (type.startsWith('video/')) return 'video'
  if (type.startsWith('audio/')) return 'audio'
  return 'document'
}

async function fileToBase64(file: File): Promise<string> {
  const bytes = new Uint8Array(await file.arrayBuffer())
  let binary = ''
  const chunkSize = 0x8000
  for (let offset = 0; offset < bytes.length; offset += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(offset, offset + chunkSize))
  }
  return btoa(binary)
}
