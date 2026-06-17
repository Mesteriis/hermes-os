import type { TelegramProviderWriteCommand } from '../types/telegram'

export type TelegramCommandAuditTone = 'neutral' | 'progress' | 'success' | 'warning' | 'danger'

export type TelegramCommandAuditState = {
  label: string
  detail: string
  tone: TelegramCommandAuditTone
  is_dead_letter: boolean
}

function providerStateString(
  command: TelegramProviderWriteCommand,
  key: string
): string | null {
  const value = command.provider_state[key]
  return typeof value === 'string' && value.trim().length > 0 ? value : null
}

function providerStateBoolean(
  command: TelegramProviderWriteCommand,
  key: string
): boolean | null {
  const value = command.provider_state[key]
  return typeof value === 'boolean' ? value : null
}

function payloadString(
  command: TelegramProviderWriteCommand,
  key: string
): string | null {
  const value = command.payload[key]
  return typeof value === 'string' && value.trim().length > 0 ? value : null
}

function payloadNumber(
  command: TelegramProviderWriteCommand,
  key: string
): number | null {
  const value = command.payload[key]
  return typeof value === 'number' ? value : null
}

function providerStateNumber(
  command: TelegramProviderWriteCommand,
  key: string
): number | null {
  const value = command.provider_state[key]
  return typeof value === 'number' ? value : null
}

function folderId(command: TelegramProviderWriteCommand): number | null {
  return providerStateNumber(command, 'provider_folder_id') ?? payloadNumber(command, 'provider_folder_id')
}

function textLengthLabel(value: string): string {
  return `${value.length} chars`
}

function editMismatchDetail(command: TelegramProviderWriteCommand): string | null {
  if (command.command_kind !== 'edit' || command.reconciliation_status !== 'mismatch') return null

  const expectedBody = providerStateString(command, 'expected_body_text')
  const observedBody = providerStateString(command, 'observed_body_text')
  if (expectedBody && observedBody) {
    return `Provider mismatch · expected ${textLengthLabel(expectedBody)}, observed ${textLengthLabel(observedBody)}`
  }
  return 'Provider mismatch observed'
}

function reactionMismatchDetail(command: TelegramProviderWriteCommand): string | null {
  if (
    (command.command_kind !== 'react' && command.command_kind !== 'unreact') ||
    command.reconciliation_status !== 'mismatch'
  ) {
    return null
  }

  const reactionEmoji =
    providerStateString(command, 'reaction_emoji') ??
    payloadString(command, 'reaction_emoji')
  const observedIsChosen = providerStateBoolean(command, 'observed_is_chosen')
  if (!reactionEmoji) return 'Provider mismatch observed'
  if (observedIsChosen === true) {
    return `Provider mismatch · reaction ${reactionEmoji} is still present`
  }
  if (observedIsChosen === false) {
    return `Provider mismatch · reaction ${reactionEmoji} is still absent`
  }
  return `Provider mismatch · reaction ${reactionEmoji}`
}

function pinMismatchDetail(command: TelegramProviderWriteCommand): string | null {
  if (
    (command.command_kind !== 'pin' && command.command_kind !== 'unpin') ||
    command.reconciliation_status !== 'mismatch'
  ) {
    return null
  }

  const observedIsPinned = providerStateBoolean(command, 'observed_is_pinned')
  if (observedIsPinned === true) {
    return command.provider_message_id
      ? 'Provider mismatch · message is still pinned'
      : 'Provider mismatch · chat is still pinned'
  }
  if (observedIsPinned === false) {
    return command.provider_message_id
      ? 'Provider mismatch · message is still unpinned'
      : 'Provider mismatch · chat is still unpinned'
  }
  return 'Provider mismatch observed'
}

function chatLifecycleMismatchDetail(command: TelegramProviderWriteCommand): string | null {
  const pinMismatch = pinMismatchDetail(command)
  if (pinMismatch) return pinMismatch

  if (command.command_kind === 'mark_unread' && command.reconciliation_status === 'mismatch') {
    const observedIsMarkedUnread = providerStateBoolean(command, 'observed_is_marked_as_unread')
    if (observedIsMarkedUnread === true) return 'Provider mismatch · chat is still unread'
    if (observedIsMarkedUnread === false) return 'Provider mismatch · chat is still read'
    return 'Provider mismatch observed'
  }

  if (
    (command.command_kind === 'archive' || command.command_kind === 'unarchive') &&
    command.reconciliation_status === 'mismatch'
  ) {
    const observedIsArchived = providerStateBoolean(command, 'observed_is_archived')
    if (observedIsArchived === true) return 'Provider mismatch · chat is still archived'
    if (observedIsArchived === false) return 'Provider mismatch · chat is still unarchived'
    return 'Provider mismatch observed'
  }

  if (
    (command.command_kind === 'mute' || command.command_kind === 'unmute') &&
    command.reconciliation_status === 'mismatch'
  ) {
    const observedIsMuted = providerStateBoolean(command, 'observed_is_muted')
    if (observedIsMuted === true) return 'Provider mismatch · chat is still muted'
    if (observedIsMuted === false) return 'Provider mismatch · chat is still unmuted'
    return 'Provider mismatch observed'
  }

  return null
}

function messageLifecycleDetail(command: TelegramProviderWriteCommand): string | null {
  const mismatch = editMismatchDetail(command)
  if (mismatch) return mismatch
  const reactionMismatch = reactionMismatchDetail(command)
  if (reactionMismatch) return reactionMismatch
  const chatMismatch = chatLifecycleMismatchDetail(command)
  if (chatMismatch) return chatMismatch

  switch (command.command_kind) {
    case 'edit': {
      const observedBody = providerStateString(command, 'body_text')
      if (observedBody) return `Provider text observed · ${textLengthLabel(observedBody)}`
      const targetBody = payloadString(command, 'new_text')
      if (targetBody) return `Target text · ${textLengthLabel(targetBody)}`
      return null
    }
    case 'delete': {
      if (providerStateBoolean(command, 'is_deleted') === true) {
        return 'Provider delete observed'
      }
      const reasonClass = payloadString(command, 'reason_class')
      return reasonClass ? `Delete requested · ${reasonClass}` : 'Delete requested'
    }
    case 'restore_visibility': {
      if (command.status === 'completed') return 'Visibility restored locally'
      const reason = payloadString(command, 'reason')
      return reason ? `Visibility restore requested · ${reason}` : 'Visibility restore requested'
    }
    case 'mark_unread': {
      const isMarkedUnread = providerStateBoolean(command, 'is_marked_as_unread')
      if (isMarkedUnread === true) return 'Marked unread on provider'
      if (isMarkedUnread === false) return 'Marked read on provider'
      return 'Mark chat unread'
    }
    case 'pin':
    case 'unpin': {
      const isPinned = providerStateBoolean(command, 'is_pinned')
      if (isPinned === true) return 'Pinned on provider'
      if (isPinned === false) return 'Unpinned on provider'
      return command.command_kind === 'pin' ? 'Pin requested' : 'Unpin requested'
    }
    case 'archive':
    case 'unarchive': {
      const isArchived = providerStateBoolean(command, 'is_archived')
      if (isArchived === true) return 'Archived on provider'
      if (isArchived === false) return 'Restored from archive on provider'
      return command.command_kind === 'archive' ? 'Archive requested' : 'Unarchive requested'
    }
    case 'mute':
    case 'unmute': {
      const isMuted = providerStateBoolean(command, 'is_muted')
      if (isMuted === true) return 'Muted on provider'
      if (isMuted === false) return 'Unmuted on provider'
      return command.command_kind === 'mute' ? 'Mute requested' : 'Unmute requested'
    }
    case 'folder_add':
    case 'folder_remove': {
      const providerFolderId = folderId(command)
      if (providerFolderId !== null && command.status === 'completed') {
        return command.command_kind === 'folder_add'
          ? `Folder ${providerFolderId} observed on provider`
          : `Folder ${providerFolderId} removal observed on provider`
      }
      if (providerFolderId !== null) {
        return command.command_kind === 'folder_add'
          ? `Add to folder ${providerFolderId}`
          : `Remove from folder ${providerFolderId}`
      }
      return command.command_kind === 'folder_add' ? 'Add to folder' : 'Remove from folder'
    }
    case 'react':
    case 'unreact': {
      const reactionEmoji =
        providerStateString(command, 'reaction_emoji') ??
        payloadString(command, 'reaction_emoji')
      const isChosen = providerStateBoolean(command, 'is_chosen')
      if (reactionEmoji && isChosen === true) {
        return `Reaction ${reactionEmoji} present on provider`
      }
      if (reactionEmoji && isChosen === false) {
        return `Reaction ${reactionEmoji} absent on provider`
      }
      if (reactionEmoji) {
        return command.command_kind === 'react'
          ? `Add reaction ${reactionEmoji}`
          : `Remove reaction ${reactionEmoji}`
      }
      return command.command_kind === 'react' ? 'Add reaction' : 'Remove reaction'
    }
    default:
      return null
  }
}

function executingCommandDetail(command: TelegramProviderWriteCommand): string {
  const participantLifecycle = participantLifecycleDetail(command)
  if (participantLifecycle) return participantLifecycle

  const readProgress = markReadProgress(command)
  if (readProgress) return readProgress

  const lifecycleDetail = messageLifecycleDetail(command)
  if (lifecycleDetail) return lifecycleDetail

  const progressDetail = providerStateString(command, 'progress_detail')
  if (progressDetail) return progressDetail

  const uploadPhase = providerStateString(command, 'upload_phase')
  if (uploadPhase === 'dispatching_to_provider') return 'Uploading local media to Telegram'

  if (command.reconciliation_status === 'awaiting_provider') {
    return 'Awaiting provider-observed state'
  }

  return telegramCommandRetrySummary(command)
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

export function telegramCommandSubject(command: TelegramProviderWriteCommand): string {
  if (command.command_kind === 'mark_read') {
    return command.provider_message_id
      ? `Read through ${command.provider_message_id}`
      : 'Mark chat read'
  }
  if (command.command_kind === 'mark_unread') {
    return 'Mark chat unread'
  }
  if (command.command_kind === 'edit') {
    return 'Edit message'
  }
  if (command.command_kind === 'delete') {
    return 'Delete message'
  }
  if (command.command_kind === 'restore_visibility') {
    return 'Restore message visibility'
  }
  if (command.command_kind === 'pin') {
    return command.provider_message_id ? 'Pin message' : 'Pin chat'
  }
  if (command.command_kind === 'unpin') {
    return command.provider_message_id ? 'Unpin message' : 'Unpin chat'
  }
  if (command.command_kind === 'archive') {
    return 'Archive chat'
  }
  if (command.command_kind === 'unarchive') {
    return 'Unarchive chat'
  }
  if (command.command_kind === 'mute') {
    return 'Mute chat'
  }
  if (command.command_kind === 'unmute') {
    return 'Unmute chat'
  }
  if (command.command_kind === 'folder_add') {
    const providerFolderId = folderId(command)
    return providerFolderId !== null
      ? `Add chat to folder ${providerFolderId}`
      : 'Add chat to folder'
  }
  if (command.command_kind === 'folder_remove') {
    const providerFolderId = folderId(command)
    return providerFolderId !== null
      ? `Remove chat from folder ${providerFolderId}`
      : 'Remove chat from folder'
  }
  if (command.command_kind === 'react' || command.command_kind === 'unreact') {
    const reactionEmoji =
      providerStateString(command, 'reaction_emoji') ??
      payloadString(command, 'reaction_emoji')
    if (!reactionEmoji) {
      return command.command_kind === 'react' ? 'Add reaction' : 'Remove reaction'
    }
    return command.command_kind === 'react'
      ? `Add reaction ${reactionEmoji}`
      : `Remove reaction ${reactionEmoji}`
  }
  return command.provider_message_id ?? command.provider_chat_id
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
    const mismatchDetail = messageLifecycleDetail(command)
    return {
      label: 'Failed',
      detail: mismatchDetail ?? command.last_error ?? 'Provider write failed',
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
      detail: executingCommandDetail(command),
      tone: 'progress',
      is_dead_letter: false,
    }
  }

  if (command.status === 'completed') {
    const participantLifecycle = participantLifecycleDetail(command)
    const readProgress = markReadProgress(command)
    const lifecycleDetail = messageLifecycleDetail(command)
    return {
      label: 'Completed',
      detail:
        participantLifecycle ??
        readProgress ??
        lifecycleDetail ??
        (command.completed_at ? 'Provider write completed' : 'Completion recorded'),
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
    detail:
      markReadProgress(command) ??
      messageLifecycleDetail(command) ??
      telegramCommandRetrySummary(command),
    tone: 'neutral',
    is_dead_letter: false,
  }
}

function markReadProgress(command: TelegramProviderWriteCommand): string | null {
  if (command.command_kind !== 'mark_read') return null
  const observed = providerStateString(command, 'last_read_inbox_message_id')
  if (observed) return `Read through ${observed}`
  if (command.provider_message_id) return `Read through ${command.provider_message_id}`
  return null
}

function participantLifecycleDetail(command: TelegramProviderWriteCommand): string | null {
  if (command.command_kind !== 'join' && command.command_kind !== 'leave') return null

  const membershipState = providerStateString(command, 'membership_state')
  if (command.command_kind === 'join' && membershipState === 'present') {
    return 'Joined chat'
  }

  if (command.command_kind === 'leave') {
    if (membershipState === 'absent_exhaustive') {
      return 'Left chat (confirmed by full provider roster)'
    }
    if (membershipState === 'left' || membershipState === 'banned') {
      return 'Left chat'
    }
  }

  return null
}
