import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  TelegramAutomationPolicyListResponse,
  TelegramAutomationTemplateListResponse,
  TelegramSendDryRunRequest,
  TelegramSendDryRunResponse,
} from '../types/automation'

export async function fetchTelegramAutomationPolicies(): Promise<TelegramAutomationPolicyListResponse> {
  return ApiClient.instance.get<TelegramAutomationPolicyListResponse>(
    '/api/v1/policies',
    'Telegram automation policy request failed'
  )
}

export async function fetchTelegramAutomationTemplates(): Promise<TelegramAutomationTemplateListResponse> {
  return ApiClient.instance.get<TelegramAutomationTemplateListResponse>(
    '/api/v1/policies/templates',
    'Telegram automation template request failed'
  )
}

export async function runTelegramSendDryRun(
  request: TelegramSendDryRunRequest
): Promise<TelegramSendDryRunResponse> {
  return ApiClient.instance.post<TelegramSendDryRunResponse>(
    '/api/v1/policies/telegram-send/dry-run',
    request,
    'Telegram send dry-run failed'
  )
}
