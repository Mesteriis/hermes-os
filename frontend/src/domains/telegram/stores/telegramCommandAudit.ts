import type { TelegramProviderWriteCommand } from '../types/telegram'

export type TelegramCommandAuditTone = 'neutral' | 'progress' | 'success' | 'warning' | 'danger'

export type TelegramCommandAuditState = {
  label: string
  detail: string
  tone: TelegramCommandAuditTone
  is_dead_letter: boolean
}

function retryBudget(command: TelegramProviderWriteCommand): { used: number; max: number } {
  return {
    used: Math.max(0, command.retry_count),
    max: Math.max(0, command.max_retries),
  }
}

export function telegramCommandRetrySummary(command: TelegramProviderWriteCommand): string {
  const { used, max } = retryBudget(command)
  if (max === 0) return 'No retry budget'
  return `${Math.min(used, max)}/${max} retries used`
}

export function isTelegramCommandDeadLetter(command: TelegramProviderWriteCommand): boolean {
  const { used, max } = retryBudget(command)
  return command.status === 'dead_letter' || (command.status === 'failed' && max > 0 && used >= max)
}

export function telegramCommandAuditState(command: TelegramProviderWriteCommand): TelegramCommandAuditState {
  if (isTelegramCommandDeadLetter(command)) {
    return {
      label: 'Dead-lettered',
      detail: command.last_error ?? 'Retry budget exhausted',
      tone: 'danger',
      is_dead_letter: true,
    }
  }

  if (command.status === 'failed') {
    return {
      label: 'Failed',
      detail: command.last_error ?? 'Provider write failed',
      tone: 'warning',
      is_dead_letter: false,
    }
  }

  if (command.status === 'retrying') {
    return {
      label: 'Retrying',
      detail: telegramCommandRetrySummary(command),
      tone: 'warning',
      is_dead_letter: false,
    }
  }

  if (command.status === 'executing') {
    return {
      label: 'Executing',
      detail: command.reconciliation_status === 'awaiting_provider'
        ? 'Awaiting provider-observed state'
        : telegramCommandRetrySummary(command),
      tone: 'progress',
      is_dead_letter: false,
    }
  }

  if (command.status === 'completed') {
    return {
      label: 'Completed',
      detail: command.completed_at ? 'Provider write completed' : 'Completion recorded',
      tone: 'success',
      is_dead_letter: false,
    }
  }

  if (command.status === 'cancelled') {
    return {
      label: 'Cancelled',
      detail: telegramCommandRetrySummary(command),
      tone: 'neutral',
      is_dead_letter: false,
    }
  }

  return {
    label: 'Queued',
    detail: telegramCommandRetrySummary(command),
    tone: 'neutral',
    is_dead_letter: false,
  }
}
