import type {
  TelegramCapabilitiesResponse,
  TelegramCapabilityState,
  TelegramOperationCapability
} from '../types/telegram'

export type TelegramComposerCapabilityHint = {
  operation: string
  status: TelegramCapabilityState | 'unknown'
  reason: string
  title: string
  summary: string
}

function findCapability(
  capabilities: TelegramCapabilitiesResponse | null | undefined,
  operation: string
): TelegramOperationCapability | null {
  return capabilities?.capabilities.find((capability) => capability.operation === operation) ?? null
}

function composerCapabilityHint(
  capabilities: TelegramCapabilitiesResponse | null | undefined,
  operation: string,
  fallbackReason: string
): TelegramComposerCapabilityHint {
  const capability = findCapability(capabilities, operation)
  const status = capability?.status ?? 'unknown'
  const reason = capability?.reason.trim() || fallbackReason

  return {
    operation,
    status,
    reason,
    title: `${operation}: ${status} - ${reason}`,
    summary: `${operation}: ${status} - ${reason}`
  }
}

export function telegramComposerMediaCapabilityHint(
  capabilities: TelegramCapabilitiesResponse | null | undefined
): TelegramComposerCapabilityHint {
  return composerCapabilityHint(
    capabilities,
    'messages.send_media',
    'Media upload/send capability is not available from the selected account contract.'
  )
}

export function telegramComposerVoiceCapabilityHint(
  capabilities: TelegramCapabilitiesResponse | null | undefined
): TelegramComposerCapabilityHint {
  return composerCapabilityHint(
    capabilities,
    'voice.record_send',
    'Voice recording/send capability is not available from the selected account contract.'
  )
}
