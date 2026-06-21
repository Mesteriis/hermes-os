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
    queryKey: computed(() => ['communications', 'messages', toValue(messageId), 'reply-chain']),
    queryFn: () => fetchTelegramReplyChain(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramForwardChainQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramForwardChainResponse>({
    queryKey: computed(() => ['communications', 'messages', toValue(messageId), 'forward-chain']),
    queryFn: () => fetchTelegramForwardChain(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}
