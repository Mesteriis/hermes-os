import { useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchTelegramForwardChain,
  fetchTelegramReplyChain,
} from '../api/telegramLifecycle'
import type {
  TelegramForwardChainResponse,
  TelegramReplyChainResponse,
} from '../types/telegram'

export function useTelegramReplyChainQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramReplyChainResponse>({
    queryKey: computed(() => ['telegram', 'reply-chain', toValue(messageId)]),
    queryFn: () => fetchTelegramReplyChain(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramForwardChainQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramForwardChainResponse>({
    queryKey: computed(() => ['telegram', 'forward-chain', toValue(messageId)]),
    queryFn: () => fetchTelegramForwardChain(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}
