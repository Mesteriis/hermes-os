import type { TelegramSendDryRunRequest } from '../types/automation'

export type TelegramRuntimeSetupForm = {
  accountId: string
  apiHash: string
  apiId: string
  botToken: string
  displayName: string
  externalAccountId: string
  providerKind: string
  qrAuthorized: boolean
  sessionEncryptionKey: string
  tdlibDataPath: string
}

export type TelegramAccountSetupRequest = {
  account_id: string
  provider_kind: string
  display_name: string
  external_account_id: string
  api_id?: number
  api_hash?: string
  bot_token?: string
  session_encryption_key?: string
  tdlib_data_path?: string
  qr_authorized?: boolean
  transcription_enabled: boolean
}

export const TELEGRAM_RUNTIME_CHAT_SYNC_CHUNK_SIZE = 200
export const TELEGRAM_RUNTIME_PROVIDER_SEARCH_CHUNK_SIZE = 50
export const TELEGRAM_RUNTIME_COMMANDS_PAGE_SIZE = 50
export const TELEGRAM_RUNTIME_CALLS_PAGE_SIZE = 50

export function createTelegramRuntimeSetupForm(): TelegramRuntimeSetupForm {
  return {
    accountId: '', apiHash: '', apiId: '', botToken: '', displayName: '', externalAccountId: '',
    providerKind: 'telegram_user', qrAuthorized: false, sessionEncryptionKey: '', tdlibDataPath: '',
  }
}

export function buildTelegramAccountSetupRequest(
  form: TelegramRuntimeSetupForm
): { request: TelegramAccountSetupRequest } | { error: string } {
  const accountId = form.accountId.trim()
  const displayName = form.displayName.trim()
  const externalAccountId = form.externalAccountId.trim()
  if (!accountId || !displayName || !externalAccountId) return { error: 'required' }

  const apiIdText = form.apiId.trim()
  const apiId = apiIdText ? Number.parseInt(apiIdText, 10) : undefined
  if (apiId != null && !Number.isFinite(apiId)) return { error: 'api_id_invalid' }

  return {
    request: {
      account_id: accountId,
      provider_kind: form.providerKind,
      display_name: displayName,
      external_account_id: externalAccountId,
      api_id: apiId,
      api_hash: form.apiHash || undefined,
      bot_token: form.botToken || undefined,
      session_encryption_key: form.sessionEncryptionKey || undefined,
      tdlib_data_path: form.tdlibDataPath || undefined,
      qr_authorized: form.qrAuthorized,
      transcription_enabled: false,
    },
  }
}

export function parseTelegramDryRunVariables(value: string): Record<string, string> {
  const parsed: unknown = JSON.parse(value)
  if (!isRecord(parsed)) {
    throw new Error('Dry-run variables must be a JSON object.')
  }
  const variables: Record<string, string> = {}
  for (const [key, item] of Object.entries(parsed)) {
    if (typeof item !== 'string') throw new Error(`Dry-run variable ${key} must be a string.`)
    variables[key] = item
  }
  return variables
}

export function canRunTelegramDryRun(policyId: string, providerChatId: string): boolean {
  return policyId.trim().length > 0 && providerChatId.trim().length > 0
}

export function canTriggerTelegramProviderSearch(accountId: string | null, query: string): boolean {
  return Boolean(accountId) && query.trim().length > 0
}

export function buildTelegramDryRunRequest(
  policyId: string,
  providerChatId: string,
  variables: Record<string, string>,
  commandId: string
): TelegramSendDryRunRequest {
  return { command_id: commandId, policy_id: policyId, provider_chat_id: providerChatId, variables }
}

export function telegramRuntimeCommandId(): string {
  return crypto.randomUUID()
}

export function buildTelegramSyncChatsRequest(accountId: string): { account_id: string; limit: number } {
  return { account_id: accountId, limit: TELEGRAM_RUNTIME_CHAT_SYNC_CHUNK_SIZE }
}

export function buildTelegramProviderSearchRequest(
  accountId: string,
  query: string
): { account_id: string; q: string; limit: number } {
  return { account_id: accountId, q: query, limit: TELEGRAM_RUNTIME_PROVIDER_SEARCH_CHUNK_SIZE }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}
