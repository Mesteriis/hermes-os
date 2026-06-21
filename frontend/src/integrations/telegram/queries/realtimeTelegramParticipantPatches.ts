import type { TelegramChatMember } from '../types/telegram'
import { isRecord, storedEventEnvelope, stringValue } from '../../../shared/communications/queries/realtimePatchShared'

type TelegramChatMembersPage = {
  items: TelegramChatMember[]
  next_cursor: string | null
}

type TelegramChatMembersInfiniteData = {
  pages: TelegramChatMembersPage[]
  pageParams: unknown[]
}

export type TelegramParticipantPatchQueryClient = {
  getQueriesData?: <TData>(filters: { queryKey: readonly unknown[] }) => Array<
    [readonly unknown[], TData | undefined]
  >
  setQueryData?: <TData>(
    queryKey: readonly unknown[],
    updater: TData | ((data: TData | undefined) => TData | undefined)
  ) => unknown
}

export function applyTelegramParticipantRealtimePatch(
  eventData: string,
  queryClient: TelegramParticipantPatchQueryClient
): boolean {
  const { getQueriesData, setQueryData } = queryClient
  if (!getQueriesData || !setQueryData) return false

  const envelope = storedEventEnvelope(eventData)
  const event = envelope?.event
  if (event?.event_type !== 'telegram.participant.updated') return false
  const payload = isRecord(event.payload) ? event.payload : undefined
  const telegramChatId = stringValue(payload?.telegram_chat_id)
  const participant = telegramChatMemberSnapshot(payload?.participant)
  if (!telegramChatId || !participant) return false

  let patched = false
  for (const [queryKey, data] of getQueriesData<TelegramChatMember[] | TelegramChatMembersInfiniteData>({
    queryKey: ['communications', 'telegram', 'chat-members']
  })) {
    if (queryKey[3] !== telegramChatId) continue
    const updated = patchParticipantQuery(queryKey, data, participant)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }
  return patched
}

function telegramChatMemberSnapshot(value: unknown): TelegramChatMember | null {
  if (!isRecord(value)) return null
  const providerMemberId = stringValue(value.provider_member_id) ?? stringValue(value.sender_id)
  if (!providerMemberId) return null
  return {
    sender_id: stringValue(value.sender_id) ?? providerMemberId,
    sender_display_name: stringValue(value.sender_display_name),
    message_count: typeof value.message_count === 'number' ? value.message_count : 0,
    last_message_at: stringValue(value.last_message_at),
    source: telegramMemberSource(value.source),
    provider_member_id: providerMemberId,
    username: stringValue(value.username),
    role: stringValue(value.role),
    status: stringValue(value.status),
    is_admin: value.is_admin === true,
    is_owner: value.is_owner === true,
    permissions: isRecord(value.permissions) ? value.permissions : {},
    observed_at: stringValue(value.observed_at),
  }
}

function telegramMemberSource(value: unknown): TelegramChatMember['source'] {
  return value === 'tdlib' || value === 'bot_api' || value === 'message_heuristic'
    ? value
    : 'tdlib'
}

function patchParticipantQuery(
  queryKey: readonly unknown[],
  data: TelegramChatMember[] | TelegramChatMembersInfiniteData | undefined,
  participant: TelegramChatMember
): TelegramChatMember[] | TelegramChatMembersInfiniteData | undefined {
  if (!data) return data

  const query = typeof queryKey[5] === 'string' ? queryKey[5].trim().toLowerCase() : ''
  const role = typeof queryKey[6] === 'string' ? queryKey[6].trim().toLowerCase() : ''

  if (Array.isArray(data)) {
    return patchParticipantCollection(data, participant, query, role)
  }

  if (!isInfiniteData(data)) return data
  const nextPages = data.pages.map((page) => ({
    ...page,
    items: patchParticipantCollection(page.items, participant, query, role) ?? page.items,
  }))
  return nextPages.some((page, index) => page.items !== data.pages[index]?.items)
    ? { ...data, pages: nextPages }
    : data
}

function patchParticipantCollection(
  members: TelegramChatMember[] | undefined,
  participant: TelegramChatMember,
  query: string,
  role: string
): TelegramChatMember[] | undefined {
  if (!members) return members
  if (participantIsInactive(participant)) {
    return members.filter((member) => member.provider_member_id !== participant.provider_member_id)
  }
  const participantMatches = participantMatchesFilters(participant, query, role)
  const existingIndex = members.findIndex(
    (member) => member.provider_member_id === participant.provider_member_id
  )
  if (!participantMatches) {
    if (existingIndex < 0) return members
    return members.filter((member) => member.provider_member_id !== participant.provider_member_id)
  }
  if (existingIndex < 0) return [participant, ...members]
  return members.map((member, index) => (index === existingIndex ? participant : member))
}

function participantIsInactive(participant: TelegramChatMember): boolean {
  const status = (participant.status ?? '').trim().toLowerCase()
  const role = (participant.role ?? '').trim().toLowerCase()
  return (
    status === 'left' ||
    status === 'banned' ||
    status === 'absent_exhaustive' ||
    role === 'left' ||
    role === 'banned'
  )
}

function participantMatchesFilters(
  participant: TelegramChatMember,
  query: string,
  role: string
): boolean {
  if (role && (participant.role ?? '').trim().toLowerCase() !== role) return false
  if (!query) return true

  return [
    participant.sender_display_name ?? '',
    participant.sender_id,
    participant.provider_member_id,
    participant.username ?? '',
    participant.role ?? '',
    participant.status ?? '',
    participant.source,
  ]
    .join(' ')
    .toLowerCase()
    .includes(query)
}

function isInfiniteData(value: unknown): value is TelegramChatMembersInfiniteData {
  return isRecord(value) && Array.isArray(value.pages) && Array.isArray(value.pageParams)
}
