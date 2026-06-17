import type { TelegramRuntimeStatus } from '../types/telegram'

export function telegramRuntimeCommandTarget(status: TelegramRuntimeStatus | null): string | null {
  if (!status?.last_command_status) return null

  if (status.last_command_kind === 'mark_read' && status.last_command_message_id) {
    return `Read through ${status.last_command_message_id}`
  }

  return status.last_command_message_id
    ?? status.last_command_telegram_chat_id
    ?? status.last_command_provider_chat_id
    ?? status.last_command_id
    ?? null
}

