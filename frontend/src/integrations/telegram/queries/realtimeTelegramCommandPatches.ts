import { isRecord, numberValue, stringValue } from '../../../domains/communications/queries/realtimePatchShared'
import type { TelegramProviderWriteCommand } from '../types/telegram'
import type { TelegramEventPayload } from './realtimeTelegramPatchShared'

export function patchTelegramCommandList(
  queryKey: readonly unknown[],
  commands: TelegramProviderWriteCommand[] | undefined,
  eventType: string,
  payload: TelegramEventPayload | undefined
): TelegramProviderWriteCommand[] | undefined {
  if (
    !commands ||
    (
      eventType !== 'telegram.command.status_changed' &&
      eventType !== 'telegram.command.reconciled' &&
      eventType !== 'telegram.media.upload.started' &&
      eventType !== 'telegram.media.upload.progress'
    ) ||
    !payload
  ) return commands
  if (queryKey[0] !== 'integrations' || queryKey[1] !== 'telegram' || queryKey[2] !== 'commands') return commands

  const queryAccountId = typeof queryKey[3] === 'string' && queryKey[3] !== 'none' ? queryKey[3] : null
  const queryProviderChatId =
    typeof queryKey[5] === 'string' && queryKey[5] !== 'all' ? queryKey[5] : null
  const queryProviderMessageId =
    typeof queryKey[6] === 'string' && queryKey[6] !== 'all' ? queryKey[6] : null
  const queryCommandKinds =
    typeof queryKey[7] === 'string' && queryKey[7] !== 'all'
      ? new Set(queryKey[7].split('|').filter((value) => value.length > 0))
      : null
  const payloadAccountId = stringValue(payload.account_id)
  if (queryAccountId && payloadAccountId && payloadAccountId !== queryAccountId) return commands
  const commandId = stringValue(payload.command_id)
  const newStatus = stringValue(payload.status)
  if (!commandId || !newStatus) return commands

  const payloadProviderChatId = stringValue(payload.provider_chat_id)
  if (queryProviderChatId && payloadProviderChatId && payloadProviderChatId !== queryProviderChatId) {
    return commands
  }
  const payloadProviderMessageId =
    stringValue(payload.provider_message_id) ?? stringValue(payload.message_id)
  if (
    queryProviderMessageId &&
    payloadProviderMessageId &&
    payloadProviderMessageId !== queryProviderMessageId
  ) {
    return commands
  }
  const payloadCommandKind = insertedCommandKind(eventType, payload)
  if (queryCommandKinds && payloadCommandKind && !queryCommandKinds.has(payloadCommandKind)) {
    return commands
  }

  const matchIndex = commands.findIndex((command) => command.command_id === commandId)
  if (matchIndex < 0) {
    if (!payloadAccountId) return commands
    return insertCommand(
      queryKey,
      commands,
      payloadAccountId,
      commandId,
      newStatus,
      eventType,
      payload
    )
  }

  const matchedCommand = commands[matchIndex]
  if (queryAccountId && matchedCommand.account_id !== queryAccountId) return commands

  const nextCommand = {
    ...matchedCommand,
    status: newStatus as TelegramProviderWriteCommand['status'],
    retry_count: numberValue(payload.retry_count) ?? matchedCommand.retry_count,
    max_retries: numberValue(payload.max_retries) ?? matchedCommand.max_retries,
    last_error: normalizeNullableString(payload.last_error, matchedCommand.last_error),
    result_payload: recordValue(payload.result_payload) ?? matchedCommand.result_payload,
    next_attempt_at: normalizeNullableString(payload.next_attempt_at, matchedCommand.next_attempt_at),
    last_attempt_at: normalizeNullableString(payload.last_attempt_at, matchedCommand.last_attempt_at),
    provider_observed_at: normalizeNullableString(payload.provider_observed_at, matchedCommand.provider_observed_at),
    provider_state: recordValue(payload.provider_state) ?? matchedCommand.provider_state,
    reconciliation_status:
      stringValue(payload.reconciliation_status) ?? matchedCommand.reconciliation_status,
    reconciled_at: normalizeNullableString(payload.reconciled_at, matchedCommand.reconciled_at),
    dead_lettered_at: normalizeNullableString(payload.dead_lettered_at, matchedCommand.dead_lettered_at),
    completed_at: normalizeNullableString(payload.completed_at, matchedCommand.completed_at),
    updated_at: new Date().toISOString(),
  } satisfies TelegramProviderWriteCommand

  return commands.map((command, index) => index === matchIndex ? nextCommand : command)
}

function normalizeNullableString(value: unknown, fallback: string | null): string | null {
  if (value === null) return null
  return stringValue(value) ?? fallback
}

function recordValue(value: unknown): Record<string, unknown> | null {
  return isRecord(value) ? value : null
}

function insertCommand(
  queryKey: readonly unknown[],
  commands: TelegramProviderWriteCommand[],
  accountId: string,
  commandId: string,
  status: string,
  eventType: string,
  payload: TelegramEventPayload
): TelegramProviderWriteCommand[] {
  const commandKind = insertedCommandKind(eventType, payload)
  if (!commandKind) return commands

  const now = new Date().toISOString()
  const eventPayload = insertedPayload(payload)
  const limit = typeof queryKey[4] === 'number' ? queryKey[4] : null
  const nextCommand = {
    command_id: commandId,
    account_id: accountId,
    command_kind: commandKind,
    idempotency_key: stringValue(payload.idempotency_key) ?? commandId,
    provider_chat_id: stringValue(payload.provider_chat_id) ?? '',
    provider_message_id:
      stringValue(payload.provider_message_id) ?? stringValue(payload.message_id),
    target_ref: recordValue(payload.target_ref) ?? {},
    payload: eventPayload,
    capability_state: capabilityStateValue(payload.capability_state),
    action_class: actionClassValue(payload.action_class),
    confirmation_decision: confirmationDecisionValue(payload.confirmation_decision),
    status: status as TelegramProviderWriteCommand['status'],
    retry_count: numberValue(payload.retry_count) ?? 0,
    max_retries: numberValue(payload.max_retries) ?? 0,
    last_error: normalizeNullableString(payload.last_error, null),
    result_payload: recordValue(payload.result_payload) ?? {},
    audit_metadata: recordValue(payload.audit_metadata) ?? {},
    actor_id: stringValue(payload.actor_id) ?? 'hermes-frontend',
    happened_at: stringValue(payload.happened_at) ?? now,
    next_attempt_at: normalizeNullableString(payload.next_attempt_at, null),
    last_attempt_at: normalizeNullableString(payload.last_attempt_at, null),
    locked_at: null,
    locked_by: null,
    provider_observed_at: normalizeNullableString(payload.provider_observed_at, null),
    provider_state: recordValue(payload.provider_state) ?? {},
    reconciliation_status: stringValue(payload.reconciliation_status) ?? 'not_observed',
    reconciled_at: normalizeNullableString(payload.reconciled_at, null),
    dead_lettered_at: normalizeNullableString(payload.dead_lettered_at, null),
    completed_at: normalizeNullableString(payload.completed_at, null),
    created_at: stringValue(payload.created_at) ?? now,
    updated_at: stringValue(payload.updated_at) ?? now,
  } satisfies TelegramProviderWriteCommand

  const nextCommands = [nextCommand, ...commands]
  return typeof limit === 'number' ? nextCommands.slice(0, limit) : nextCommands
}

function insertedPayload(payload: TelegramEventPayload): Record<string, unknown> {
  const explicitPayload = recordValue(payload.payload)
  if (explicitPayload) return explicitPayload

  const fallbackPayload: Record<string, unknown> = {}
  const action = stringValue(payload.action)
  const providerChatId = stringValue(payload.provider_chat_id)
  const telegramChatId = stringValue(payload.telegram_chat_id)

  if (action) fallbackPayload.action = action
  if (providerChatId) fallbackPayload.provider_chat_id = providerChatId
  if (telegramChatId) fallbackPayload.telegram_chat_id = telegramChatId

  return fallbackPayload
}

function insertedCommandKind(
  eventType: string,
  payload: TelegramEventPayload
): TelegramProviderWriteCommand['command_kind'] | null {
  if (
    eventType === 'telegram.media.upload.started' ||
    eventType === 'telegram.media.upload.progress'
  ) {
    return 'send_media'
  }

  const explicitKind = stringValue(payload.command_kind)
  if (explicitKind) return explicitKind as TelegramProviderWriteCommand['command_kind']

  const action = stringValue(payload.action)
  if (action === 'join' || action === 'leave') {
    return action as TelegramProviderWriteCommand['command_kind']
  }

  return null
}

function capabilityStateValue(value: unknown): TelegramProviderWriteCommand['capability_state'] {
  const normalized = stringValue(value)
  return normalized === 'blocked' || normalized === 'degraded' || normalized === 'unsupported'
    ? normalized
    : 'available'
}

function actionClassValue(value: unknown): TelegramProviderWriteCommand['action_class'] {
  const normalized = stringValue(value)
  return normalized === 'read' ||
    normalized === 'local_write' ||
    normalized === 'destructive' ||
    normalized === 'export' ||
    normalized === 'secret_access' ||
    normalized === 'automation'
    ? normalized
    : 'provider_write'
}

function confirmationDecisionValue(
  value: unknown
): TelegramProviderWriteCommand['confirmation_decision'] {
  const normalized = stringValue(value)
  return normalized === 'not_required' || normalized === 'rejected'
    ? normalized
    : 'confirmed'
}
