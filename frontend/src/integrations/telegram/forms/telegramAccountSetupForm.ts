import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'

export const telegramAccountSetupSchema = toTypedSchema(
  z
    .object({
      account_id: z.string().trim().min(1, 'Account ID is required'),
      provider_kind: z.enum(['telegram_user', 'telegram_bot']),
      display_name: z.string().trim().min(1, 'Display name is required'),
      external_account_id: z.string().trim().min(1, 'External account ID is required'),
      api_id: z.union([z.number().int().positive(), z.nan()]).optional(),
      api_hash: z.string().trim().optional(),
      bot_token: z.string().trim().optional(),
      session_encryption_key: z.string().trim().optional(),
      tdlib_data_path: z.string().trim().optional(),
      qr_authorized: z.boolean(),
      transcription_enabled: z.boolean(),
    })
    .superRefine((value, ctx) => {
      if (value.provider_kind === 'telegram_user') {
        if (value.qr_authorized) {
          if (!value.tdlib_data_path) {
            ctx.addIssue({
              code: z.ZodIssueCode.custom,
              path: ['tdlib_data_path'],
              message: 'TDLib data path is required for QR-authorized user accounts',
            })
          }
          return
        }
        if (!value.api_id || Number.isNaN(value.api_id)) {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            path: ['api_id'],
            message: 'API ID is required for Telegram user accounts',
          })
        }
        if (!value.api_hash) {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            path: ['api_hash'],
            message: 'API hash is required for Telegram user accounts',
          })
        }
        return
      }

      if (!value.bot_token) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          path: ['bot_token'],
          message: 'Bot token is required for Telegram bot accounts',
        })
      }
    })
)

export type TelegramAccountSetupFormValues = {
  account_id: string
  provider_kind: 'telegram_user' | 'telegram_bot'
  display_name: string
  external_account_id: string
  api_id?: number
  api_hash?: string
  bot_token?: string
  session_encryption_key?: string
  tdlib_data_path?: string
  qr_authorized: boolean
  transcription_enabled: boolean
}

export function defaultTelegramAccountSetupValues(): TelegramAccountSetupFormValues {
  return {
    account_id: '',
    provider_kind: 'telegram_user',
    display_name: '',
    external_account_id: '',
    api_id: undefined,
    api_hash: '',
    bot_token: '',
    session_encryption_key: '',
    tdlib_data_path: '',
    qr_authorized: false,
    transcription_enabled: false,
  }
}
