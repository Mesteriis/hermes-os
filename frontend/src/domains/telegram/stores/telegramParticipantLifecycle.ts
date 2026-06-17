import type { TelegramProviderWriteCommand } from '../types/telegram'

const PARTICIPANT_LIFECYCLE_KINDS = new Set(['join', 'leave'])

export function telegramParticipantLifecycleCommands(
  commands: TelegramProviderWriteCommand[],
  providerChatId: string | null,
  limit = 2
): TelegramProviderWriteCommand[] {
  if (!providerChatId) return []

  return commands
    .filter(
      (command) =>
        PARTICIPANT_LIFECYCLE_KINDS.has(command.command_kind) &&
        command.provider_chat_id === providerChatId
    )
    .sort((left, right) => lifecycleRecencyKey(right).localeCompare(lifecycleRecencyKey(left)))
    .slice(0, Math.max(0, limit))
}

export function telegramParticipantLifecycleTitle(command: TelegramProviderWriteCommand): string {
  return command.command_kind === 'leave' ? 'Leave chat' : 'Join chat'
}

function lifecycleRecencyKey(command: TelegramProviderWriteCommand): string {
  return command.updated_at || command.happened_at || command.created_at || ''
}
