import type { TelegramQrLoginStatusResponse } from '../api/telegramQrLogin'
import type { GuidedConnectionResult } from '../../stores/integrationConnectionWizard'

type Translate = (value: string) => string

export type TelegramQrWizardStatus = TelegramQrLoginStatusResponse['status'] | null | undefined

export function telegramQrNeedsPassword(status: TelegramQrWizardStatus): boolean {
  return status === 'waiting_password'
}

export function telegramQrIsReady(status: TelegramQrWizardStatus): boolean {
  return status === 'ready'
}

export function telegramQrDisplayMessage(
  status: TelegramQrWizardStatus,
  rawMessage: string | null | undefined,
  t: Translate
): string {
  switch (status) {
    case 'waiting_qr_scan':
      return t('Откройте Telegram на телефоне и отсканируйте код.')
    case 'waiting_password': {
      const hint = telegramQrHint(rawMessage)
      return hint
        ? t('Введите облачный пароль Telegram. Подсказка: ') + hint
        : t('Введите облачный пароль Telegram.')
    }
    case 'ready':
      return t('Telegram подключён. Можно выбрать сервисы.')
    case 'expired':
      return t('QR устарел. Запустите вход заново.')
    default:
      return rawMessage ?? ''
  }
}

export function shouldPollTelegramQrStatus(
  status: TelegramQrWizardStatus,
  passwordSubmitted: boolean
): boolean {
  return status === 'waiting_qr_scan' ||
    (status === 'waiting_password' && passwordSubmitted)
}

export function telegramQrResult(
  response: TelegramQrLoginStatusResponse
): Omit<GuidedConnectionResult, 'kind'> {
  return {
    title: telegramQrTitle(response.status),
    message: response.message || response.status,
    setupId: response.setup_id,
    status: response.status,
    qrSvg: response.qr_svg ?? undefined,
    qrLink: response.qr_link ?? undefined,
  }
}

function telegramQrHint(message: string | null | undefined): string | null {
  return message?.match(/Hint:\s*(.+)$/i)?.[1]?.trim() ?? null
}

function telegramQrTitle(status: TelegramQrWizardStatus): string {
  switch (status) {
    case 'waiting_qr_scan':
      return 'QR Telegram готов'
    case 'waiting_password':
      return 'Нужен облачный пароль'
    case 'ready':
      return 'Telegram подключён'
    case 'expired':
      return 'QR устарел'
    case 'failed':
    case 'runtime_unavailable':
      return 'Ошибка подключения'
    default:
      return 'Telegram'
  }
}
