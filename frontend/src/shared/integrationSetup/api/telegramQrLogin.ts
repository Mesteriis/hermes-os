import { ApiClient } from '../../../platform/api/ApiClient'

export interface TelegramQrLoginStartRequest {
  account_id: string
  display_name: string
  external_account_id: string
  transcription_enabled: boolean
}

export interface TelegramQrLoginStatusResponse {
  setup_id: string
  account_id: string
  status: string
  qr_link?: string | null
  qr_svg?: string | null
  telegram_user_id?: string | null
  telegram_username?: string | null
  suggested_account_id?: string | null
  suggested_display_name?: string | null
  suggested_external_account_id?: string | null
  expires_at?: string | null
  poll_after_ms: number
  message?: string | null
}

export interface TelegramQrLoginPasswordRequest {
  password: string
}

export async function startTelegramQrLogin(
  request: TelegramQrLoginStartRequest
): Promise<TelegramQrLoginStatusResponse> {
  return ApiClient.instance.post<TelegramQrLoginStatusResponse>(
    '/api/v1/integrations/telegram/login/qr/start',
    request,
    'Telegram QR login start failed'
  )
}

export async function fetchTelegramQrLoginStatus(
  setupId: string
): Promise<TelegramQrLoginStatusResponse> {
  return ApiClient.instance.get<TelegramQrLoginStatusResponse>(
    `/api/v1/integrations/telegram/login/qr/${encodeURIComponent(setupId)}`,
    'Telegram QR login status request failed'
  )
}

export async function submitTelegramQrLoginPassword(
  setupId: string,
  request: TelegramQrLoginPasswordRequest
): Promise<TelegramQrLoginStatusResponse> {
  return ApiClient.instance.post<TelegramQrLoginStatusResponse>(
    `/api/v1/integrations/telegram/login/qr/${encodeURIComponent(setupId)}/password`,
    request,
    'Telegram QR login password submission failed'
  )
}
