import type { TelegramMessage } from '../../types/telegram'

export type TelegramStructuredEvidenceItem = {
  key: string
  title: string
  icon: string
  lines: string[]
}

export type TelegramCustomReactionEvidence = {
  customEmojiId: string
  count: string
}

export type TelegramMessageLinkEvidence = {
  href: string
  kind: string
}

export function matchesTelegramReferenceQuery(queryValue: string, ...values: Array<string | null | undefined>): boolean {
  const query = queryValue.trim().toLowerCase()
  if (!query) return true
  return values
    .filter((value): value is string => Boolean(value))
    .join(' ')
    .toLowerCase()
    .includes(query)
}

export function buildTelegramMessageLinkEvidence(message: TelegramMessage): TelegramMessageLinkEvidence | null {
  const value = message.metadata.message_link
  if (typeof value !== 'string' || !value.startsWith('https://t.me/')) return null
  const kind = message.metadata.message_link_kind
  return { href: value, kind: typeof kind === 'string' && kind.trim() ? kind : 'provider' }
}

export function buildTelegramStructuredEvidence(
  message: TelegramMessage,
  t: (value: string) => string
): TelegramStructuredEvidenceItem[] {
  const items: TelegramStructuredEvidenceItem[] = []
  const poll = recordValue(message.metadata.telegram_poll)
  if (poll) {
    items.push({
      key: 'telegram_poll',
      title: t('Poll'),
      icon: 'tabler:chart-bar',
      lines: compactLines(stringValue(poll.question), arrayText(poll.options), poll.is_closed === true ? t('Closed') : t('Open')),
    })
  }
  const location = recordValue(message.metadata.telegram_location)
  if (location) {
    items.push({
      key: 'telegram_location',
      title: location.kind === 'venue' ? t('Venue') : t('Location'),
      icon: 'tabler:map-pin',
      lines: compactLines(stringValue(location.title), stringValue(location.address), coordinatesLine(location)),
    })
  }
  const contact = recordValue(message.metadata.telegram_contact_card)
  if (contact) {
    items.push({
      key: 'telegram_contact_card',
      title: t('Contact Card'),
      icon: 'tabler:address-book',
      lines: compactLines(
        [stringValue(contact.first_name), stringValue(contact.last_name)].filter(Boolean).join(' '),
        stringValue(contact.phone_number),
        stringValue(contact.user_id) ? `${t('User ID')} ${stringValue(contact.user_id)}` : null
      ),
    })
  }
  const joinLeave = recordValue(message.metadata.telegram_join_leave)
  if (joinLeave) {
    items.push({
      key: 'telegram_join_leave',
      title: joinLeave.action === 'leave' ? t('Leave Event') : t('Join Event'),
      icon: joinLeave.action === 'leave' ? 'tabler:user-minus' : 'tabler:user-plus',
      lines: compactLines(
        stringValue(joinLeave.source),
        stringValue(joinLeave.user_id) ? `${t('User ID')} ${stringValue(joinLeave.user_id)}` : null,
        arrayText(joinLeave.user_ids)
      ),
    })
  }
  return items
}

export function buildTelegramCustomReactionEvidence(message: TelegramMessage): TelegramCustomReactionEvidence[] {
  const summary = recordValue(message.metadata.reaction_summary)
  const customReactions = Array.isArray(summary?.custom_reactions) ? summary.custom_reactions : []
  return customReactions.flatMap((item) => {
    const reaction = recordValue(item)
    const customEmojiId = stringValue(reaction?.custom_emoji_id)
    const count = stringValue(reaction?.count)
    return customEmojiId && count ? [{ customEmojiId, count }] : []
  })
}

export function hasTelegramSourceEvidence(message: TelegramMessage, t: (value: string) => string): boolean {
  return Boolean(
    message.raw_record_id ||
    buildTelegramMessageLinkEvidence(message) ||
    buildTelegramStructuredEvidence(message, t).length > 0 ||
    buildTelegramCustomReactionEvidence(message).length > 0
  )
}

export function matchesTelegramSourceEvidence(message: TelegramMessage, t: (value: string) => string, query: string): boolean {
  if (matchesTelegramReferenceQuery(query, message.raw_record_id, message.provider_message_id)) return true
  const link = buildTelegramMessageLinkEvidence(message)
  if (link && matchesTelegramReferenceQuery(query, link.href, link.kind)) return true
  if (buildTelegramCustomReactionEvidence(message).some((item) => matchesTelegramReferenceQuery(query, item.customEmojiId, item.count))) return true
  return buildTelegramStructuredEvidence(message, t).some((item) =>
    matchesTelegramReferenceQuery(query, item.title, item.key, ...item.lines)
  )
}

function recordValue(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' && !Array.isArray(value) ? value as Record<string, unknown> : null
}

function stringValue(value: unknown): string | null {
  if (typeof value === 'string' && value.trim()) return value.trim()
  if (typeof value === 'number' && Number.isFinite(value)) return String(value)
  return null
}

function compactLines(...values: Array<string | null | undefined>): string[] {
  return values.filter((value): value is string => Boolean(value))
}

function arrayText(value: unknown): string | null {
  if (!Array.isArray(value)) return null
  const text = value
    .map((item) => recordValue(item)?.text ?? item)
    .map((item) => stringValue(item))
    .filter(Boolean)
    .join(' · ')
  return text || null
}

function coordinatesLine(value: Record<string, unknown>): string | null {
  const latitude = stringValue(value.latitude)
  const longitude = stringValue(value.longitude)
  return latitude && longitude ? `${latitude}, ${longitude}` : null
}
