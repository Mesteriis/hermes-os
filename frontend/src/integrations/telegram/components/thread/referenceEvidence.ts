import type {
  TelegramMessageTombstone,
  TelegramMessageVersion,
  TelegramProviderWriteCommand,
} from '../../types/telegram'
import { telegramCommandAuditState } from '../../stores/telegramCommandAudit'

function textLength(value: string | null | undefined): number | null {
  return typeof value === 'string' ? value.length : null
}

export function previousTelegramVersionBody(
  versions: TelegramMessageVersion[],
  currentIndex: number
): string | null {
  const nextVersion = versions[currentIndex + 1]
  return nextVersion?.body_text ?? null
}

export function summarizeTelegramVersionDelta(
  version: TelegramMessageVersion,
  previousBody: string | null
): string {
  const currentLength = textLength(version.body_text)
  const previousLength = textLength(previousBody)
  if (previousBody === null && version.body_text) {
    return `Initial captured body · ${currentLength ?? 0} chars`
  }
  if (version.body_text === previousBody) {
    return 'No textual delta recorded'
  }
  if (version.body_text === null) {
    return 'Body removed'
  }
  if (previousBody === null) {
    return `Body captured · ${currentLength ?? 0} chars`
  }
  return `${previousLength ?? 0} -> ${currentLength ?? 0} chars`
}

export function summarizeTelegramTombstoneState(tombstone: TelegramMessageTombstone): string {
  if (tombstone.is_provider_delete && tombstone.is_local_visible) {
    return 'Provider delete observed · locally restored'
  }
  if (tombstone.is_provider_delete) {
    return 'Provider delete observed'
  }
  if (tombstone.is_local_visible) {
    return 'Local visibility restored'
  }
  return 'Local visibility hidden'
}

export function summarizeTelegramCommandEvidence(command: TelegramProviderWriteCommand): string {
  const auditState = telegramCommandAuditState(command)
  return `${auditState.label} · ${auditState.detail}`
}
