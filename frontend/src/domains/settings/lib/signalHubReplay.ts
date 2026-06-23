import type {
  SignalHubReplayRequest,
  SignalHubReplayRequestCreateRequest
} from '../types/signalHub'

export type SignalHubReplaySelectorMode = 'all' | 'position' | 'time'

export interface BuildSignalHubReplayRequestInput {
  source_code?: string
  connection_id?: string
  event_pattern?: string
  selector_mode: SignalHubReplaySelectorMode
  target_consumer?: string
  target_projection?: string
  from_position?: string
  to_position?: string
  from_time?: string
  to_time?: string
}

export function buildSignalHubReplayRequest(
  input: BuildSignalHubReplayRequestInput
): SignalHubReplayRequestCreateRequest {
  const request: SignalHubReplayRequestCreateRequest = {
    source_code: normalizeOptionalText(input.source_code),
    connection_id: normalizeOptionalText(input.connection_id),
    event_pattern: normalizeOptionalText(input.event_pattern),
    target_consumer: normalizeOptionalText(input.target_consumer),
    target_projection: normalizeOptionalText(input.target_projection),
    metadata: {
      requested_from: 'settings_signal_hub',
      selector_mode: input.selector_mode
    }
  }

  if (input.selector_mode === 'position') {
    request.from_position = parseOptionalBigInt(input.from_position)
    request.to_position = parseOptionalBigInt(input.to_position)
  }

  if (input.selector_mode === 'time') {
    request.from_time = normalizeOptionalText(input.from_time)
    request.to_time = normalizeOptionalText(input.to_time)
  }

  return request
}

export function describeSignalHubReplayRequest(request: SignalHubReplayRequest): string {
  const selectors: string[] = []

  if (request.connection_id) {
    selectors.push(`connection ${request.connection_id}`)
  }

  if (request.from_position !== null || request.to_position !== null) {
    selectors.push(
      `pos ${request.from_position?.toString() ?? '...'}..${request.to_position?.toString() ?? '...'}`
    )
  }

  if (request.from_time !== null || request.to_time !== null) {
    selectors.push(`time ${request.from_time ?? '...'}..${request.to_time ?? '...'}`)
  }

  if (request.target_consumer) {
    selectors.push(`consumer ${request.target_consumer}`)
  }

  if (request.target_projection) {
    selectors.push(`projection ${request.target_projection}`)
  }

  if (selectors.length === 0) {
    return request.requested_at
  }

  selectors.push(request.requested_at)
  return selectors.join(' / ')
}

function parseOptionalBigInt(value: string | undefined): bigint | null {
  const normalized = normalizeOptionalText(value)
  if (!normalized) return null
  return BigInt(normalized)
}

function normalizeOptionalText(value: string | undefined): string | null {
  const normalized = value?.trim() ?? ''
  return normalized.length > 0 ? normalized : null
}
