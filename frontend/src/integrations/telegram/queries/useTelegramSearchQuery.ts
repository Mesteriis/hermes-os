import { useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchTelegramPinnedMessages,
  searchTelegramChats,
  searchTelegramMedia,
  searchTelegramMessages,
  searchTelegramProviderMessages,
} from '../api/telegramSearch'
import type {
  TelegramChatSearchResponse,
  TelegramMediaSearchResponse,
  TelegramMessageListResponse,
  TelegramMessageSearchResponse,
} from '../types/telegram'

export function useTelegramDialogSearchQuery(params: {
  q: MaybeRefOrGetter<string>
  accountId?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<TelegramChatSearchResponse>({
    queryKey: computed(() => [
      'communications',
      'telegram',
      'search',
      'dialogs',
      toValue(params.q).trim(),
      toValue(params.accountId) ?? 'all',
      toValue(params.limit) ?? 20,
    ]),
    queryFn: () =>
      searchTelegramChats({
        q: toValue(params.q),
        account_id: toValue(params.accountId) ?? undefined,
        limit: toValue(params.limit) ?? 20,
      }),
    enabled: computed(() => toValue(params.q).trim().length >= 2),
  })
}

export function useTelegramMessageSearchQuery(params: {
  q: MaybeRefOrGetter<string>
  accountId?: MaybeRefOrGetter<string | null | undefined>
  providerChatId?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
  providerSearchMode?: MaybeRefOrGetter<boolean>
}) {
  const shouldUseProviderSearch = computed(() => {
    const explicitMode = params.providerSearchMode
    if (explicitMode !== undefined) {
      return Boolean(toValue(explicitMode))
    }
    const accountId = toValue(params.accountId)
    return Boolean(accountId)
  })
  return useQuery<TelegramMessageSearchResponse>({
    queryKey: computed(() => [
      'communications',
      'telegram',
      'search',
      'messages',
      toValue(params.q).trim(),
      toValue(params.accountId) ?? 'all',
      shouldUseProviderSearch.value ? 'provider' : 'local',
      toValue(params.providerChatId) ?? 'all',
      toValue(params.limit) ?? 50,
    ]),
    queryFn: () => {
      const q = toValue(params.q)
      const accountId = toValue(params.accountId)?.trim()
      const providerChatId = toValue(params.providerChatId) ?? undefined
      const limit = toValue(params.limit) ?? 50
      if (shouldUseProviderSearch.value && accountId) {
        return searchTelegramProviderMessages({
          q,
          account_id: accountId,
          provider_chat_id: providerChatId,
          limit,
        })
      }
      return searchTelegramMessages({
        q,
        account_id: accountId,
        provider_chat_id: providerChatId,
        limit,
      })
    },
    enabled: computed(() =>
      toValue(params.q).trim().length >= 2 &&
      (!shouldUseProviderSearch.value || Boolean(toValue(params.accountId)))
    ),
  })
}

export function useTelegramMediaSearchQuery(params: {
  q?: MaybeRefOrGetter<string | null | undefined>
  accountId?: MaybeRefOrGetter<string | null | undefined>
  providerChatId?: MaybeRefOrGetter<string | null | undefined>
  kind?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<TelegramMediaSearchResponse>({
    queryKey: computed(() => [
      'communications',
      'telegram',
      'search',
      'media',
      toValue(params.q)?.trim() ?? '',
      toValue(params.accountId) ?? 'all',
      toValue(params.providerChatId) ?? 'all',
      toValue(params.kind) ?? 'all',
      toValue(params.limit) ?? 100,
    ]),
    queryFn: () =>
      searchTelegramMedia({
        q: toValue(params.q) ?? undefined,
        account_id: toValue(params.accountId) ?? undefined,
        provider_chat_id: toValue(params.providerChatId) ?? undefined,
        kind: toValue(params.kind) ?? undefined,
        limit: toValue(params.limit) ?? 100,
      }),
    enabled: computed(() => Boolean(toValue(params.accountId) && toValue(params.providerChatId))),
  })
}

export function useTelegramPinnedMessagesQuery(params: {
  telegramChatId?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<TelegramMessageListResponse>({
    queryKey: computed(() => [
      'communications',
      'telegram',
      'chats',
      toValue(params.telegramChatId) ?? 'none',
      'pinned-messages',
      toValue(params.limit) ?? 100,
    ]),
    queryFn: () =>
      fetchTelegramPinnedMessages({
        telegram_chat_id: toValue(params.telegramChatId) as string,
        limit: toValue(params.limit) ?? 100,
      }),
    enabled: computed(() => Boolean(toValue(params.telegramChatId))),
  })
}
