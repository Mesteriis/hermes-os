import { useMutation, useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchTelegramAutomationPolicies,
  fetchTelegramAutomationTemplates,
  runTelegramSendDryRun,
} from '../api/telegramAutomation'
import type {
  TelegramAutomationPolicy,
  TelegramAutomationTemplate,
  TelegramSendDryRunRequest,
  TelegramSendDryRunResponse,
} from '../types/automation'

export const telegramAutomationQueryKeys = {
  policies: ['telegram', 'automation', 'policies'] as const,
  templates: ['telegram', 'automation', 'templates'] as const,
}

export function useTelegramAutomationPoliciesQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<TelegramAutomationPolicy[]>({
    queryKey: computed(() => [
      ...telegramAutomationQueryKeys.policies,
      toValue(accountId) ?? 'none',
    ]),
    queryFn: async () => {
      const response = await fetchTelegramAutomationPolicies()
      const value = toValue(accountId)
      if (!value) return []
      return response.items.filter((policy) => policy.account_id === value)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useTelegramAutomationTemplatesQuery() {
  return useQuery<TelegramAutomationTemplate[]>({
    queryKey: telegramAutomationQueryKeys.templates,
    queryFn: async () => {
      const response = await fetchTelegramAutomationTemplates()
      return response.items
    },
  })
}

export function useTelegramSendDryRunMutation() {
  return useMutation<TelegramSendDryRunResponse, Error, TelegramSendDryRunRequest>({
    mutationFn: (request) => runTelegramSendDryRun(request),
  })
}
