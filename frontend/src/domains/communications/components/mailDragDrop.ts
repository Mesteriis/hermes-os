export const MAIL_MESSAGE_DRAG_TYPE = 'application/x-hermes-mail-message-selection'

export type MailMessageDragPayload = {
  kind: 'mail-message-selection'
  message_id: string
  message_ids: string[]
}

export function createMailMessageDragPayload(messageId: string, messageIds: string[] = []): string {
  const normalizedMessageId = messageId.trim()
  const normalizedMessageIds = uniqueNonBlankIds([normalizedMessageId, ...messageIds])
  return JSON.stringify({
    kind: 'mail-message-selection',
    message_id: normalizedMessageId,
    message_ids: normalizedMessageIds
  } satisfies MailMessageDragPayload)
}

export function parseMailMessageDragPayload(value: string): MailMessageDragPayload | null {
  if (!value.trim()) return null

  try {
    const parsed = JSON.parse(value) as Partial<MailMessageDragPayload>
    if (parsed.kind !== 'mail-message-selection') return null
    if (typeof parsed.message_id !== 'string' || !parsed.message_id.trim()) return null
    if (parsed.message_ids !== undefined && !validMessageIdList(parsed.message_ids)) return null
    const messageIds = uniqueNonBlankIds([
      parsed.message_id,
      ...(parsed.message_ids ?? [])
    ])
    return {
      kind: 'mail-message-selection',
      message_id: parsed.message_id.trim(),
      message_ids: messageIds
    }
  } catch {
    return null
  }
}

export function hasMailMessageDragType(types: readonly string[] | DOMStringList): boolean {
  return Array.from(types).includes(MAIL_MESSAGE_DRAG_TYPE)
}

function validMessageIdList(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === 'string' && item.trim().length > 0)
}

function uniqueNonBlankIds(values: string[]): string[] {
  return Array.from(new Set(values.map((value) => value.trim()).filter(Boolean)))
}
