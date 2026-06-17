import type { TelegramChatMember } from '../types/telegram'
import { isRecord, storedEventEnvelope, stringValue } from '../../communications/queries/realtimePatchShared'

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
  for (const [queryKey, data] of getQueriesData<TelegramChatMember[]>({
    queryKey: ['telegram', 'chat-members']
  })) {
    if (queryKey[2] !== telegramChatId) continue
    const updated = upsertParticipant(data, participant)
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

function upsertParticipant(
  members: TelegramChatMember[] | undefined,
  participant: TelegramChatMember
): TelegramChatMember[] | undefined {
  if (!members) return members
  const existingIndex = members.findIndex(
    (member) => member.provider_member_id === participant.provider_member_id
  )
  if (existingIndex < 0) return [participant, ...members]
  return members.map((member, index) => (index === existingIndex ? participant : member))
}
