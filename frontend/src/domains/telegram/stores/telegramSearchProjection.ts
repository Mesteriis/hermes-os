import type { TelegramRuntimeStatus } from '../types/telegram'

export function telegramWorkspaceSearchSourceLabel(status: TelegramRuntimeStatus | null): string {
  if (!status?.account_id) return 'Local projection search'
  if (status.fixture_runtime) return 'Fixture projection search'
  if (status.status === 'running' && status.tdjson_runtime_available) {
    return 'Provider search with projection refresh'
  }
  if (status.tdjson_runtime_available) return 'Provider search fallback to projection'
  return 'Projection search; provider runtime unavailable'
}
