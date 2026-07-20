// Historical pre-clean-room WhatsApp presentation adapter. It is not part of the active client graph.
import { TELEGRAM_REACTION_PALETTE } from '../../../shared/communications/types/telegram'
import type { TelegramReactionGroup } from '../../../shared/communications/types/telegram'
import type { TelegramChatMember } from '../../../shared/communications/types/telegramMembers'
import type { WhatsappWebMediaItem } from '../../../shared/communications/types/whatsapp'

type Translate = (key: string) => string

export type WhatsAppPanelMessage = {
  message_id: string
  raw_record_id?: string
  account_id: string
  provider_record_id?: string
  provider_message_id?: string
  provider_chat_id?: string | null
  conversation_id?: string | null
  chat_title?: string
  sender: string
  sender_display_name: string | null
  text?: string
  body_text_preview?: string | null
  occurred_at: string | null
  projected_at: string
  channel_kind?: string
  delivery_state: string
  metadata?: Record<string, unknown>
  message_metadata?: Record<string, unknown>
}

function metadataRecord(value: unknown): Record<string, unknown> | null {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null
}

function metadataString(value: unknown): string | null {
  return typeof value === 'string' && value.trim() ? value.trim() : null
}

function metadataArray(value: unknown): unknown[] {
  return Array.isArray(value) ? value : []
}

function messageMetadata(message: WhatsAppPanelMessage): Record<string, unknown> {
  return message.metadata ?? message.message_metadata ?? {}
}

export function messageTime(message: WhatsAppPanelMessage): string {
  const value = message.occurred_at ?? message.projected_at
  if (!value) return ''
  const date = new Date(value)
  return Number.isNaN(date.getTime())
    ? ''
    : new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }).format(date)
}

export function memberLabel(member: TelegramChatMember): string {
  return member.sender_display_name ?? member.username ?? member.sender_id
}

export function mediaLabel(item: WhatsappWebMediaItem): string {
  return item.file_name || item.provider_attachment_id || item.kind
}

export function mediaAttachmentId(item: WhatsappWebMediaItem): string | null {
  return item.attachment_id?.trim() || null
}

export function isPreviewableMediaItem(item: WhatsappWebMediaItem): boolean {
  const attachmentId = mediaAttachmentId(item)
  if (!attachmentId) return false
  const mime = item.mime_type?.toLowerCase() ?? ''
  const fileName = item.file_name?.toLowerCase() ?? ''
  return (
    mime.startsWith('image/') ||
    mime.startsWith('audio/') ||
    mime.startsWith('video/') ||
    mime.startsWith('text/') ||
    mime === 'application/pdf' ||
    mime === 'application/json' ||
    mime === 'application/xml' ||
    mime === 'text/csv' ||
    fileName.endsWith('.pdf')
  )
}

export function firstPreviewableMediaAttachmentId(items: WhatsappWebMediaItem[]): string | null {
  return items.find((item) => isPreviewableMediaItem(item))?.attachment_id ?? null
}

export function mediaMetaLabel(item: WhatsappWebMediaItem): string {
  const parts = [item.kind]
  if (item.mime_type) parts.push(item.mime_type)
  if (item.download_state) parts.push(item.download_state)
  return parts.join(' · ')
}

export function mediaTime(item: WhatsappWebMediaItem): string {
  const value = item.occurred_at
  if (!value) return ''
  const date = new Date(value)
  return Number.isNaN(date.getTime())
    ? ''
    : new Intl.DateTimeFormat('en', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }).format(date)
}

export function statusMessageMediaItems(
  message: WhatsAppPanelMessage,
  mediaItems: WhatsappWebMediaItem[]
): WhatsappWebMediaItem[] {
  return mediaItems.filter((item) => item.message_id === message.message_id)
}

export function statusAuthorHeadline(message: WhatsAppPanelMessage): string | null {
  const metadata = messageMetadata(message)
  return (
    metadataString(metadata.status_author_push_name) ||
    message.sender_display_name ||
    metadataString(metadata.status_author_address) ||
    message.sender
  )
}

export function statusAuthorDetail(message: WhatsAppPanelMessage): string | null {
  const metadata = messageMetadata(message)
  const parts: string[] = []
  const identityKind = metadataString(metadata.status_author_identity_kind)
  const address = metadataString(metadata.status_author_address)
  const businessProfile = metadataRecord(metadata.status_author_business_profile)
  const businessLabel = businessProfile
    ? metadataString(
      businessProfile.verified_name ??
      businessProfile.business_name ??
      businessProfile.category ??
      businessProfile.description
    )
    : null
  if (identityKind) parts.push(identityKind)
  if (address) parts.push(address)
  if (businessLabel) parts.push(businessLabel)
  return parts.length ? parts.join(' · ') : null
}

export function statusViewSummary(message: WhatsAppPanelMessage, t: Translate): string | null {
  const metadata = messageMetadata(message)
  const count =
    typeof metadata.status_view_count === 'number'
      ? metadata.status_view_count
      : null
  const lastViewer =
    metadataString(metadata.status_last_viewer_display_name) ||
    metadataString(metadata.status_last_viewer_id)
  if (count != null && lastViewer) return `${count} ${t('views')} · ${t('Last viewer')}: ${lastViewer}`
  if (count != null) return `${count} ${t('views')}`
  if (metadata.status_viewed) return t('Viewed')
  return null
}

export function statusDeletedSummary(message: WhatsAppPanelMessage, t: Translate): string | null {
  const metadata = messageMetadata(message)
  if (!metadata.status_deleted) return null
  const deletedAt = metadataString(metadata.status_deleted_at)
  if (!deletedAt) return t('Deleted')
  const date = new Date(deletedAt)
  return Number.isNaN(date.getTime())
    ? `${t('Deleted')} · ${deletedAt}`
    : `${t('Deleted')} · ${new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }).format(date)}`
}

export function statusMediaCountLabel(
  message: WhatsAppPanelMessage,
  mediaItems: WhatsappWebMediaItem[],
  t: Translate
): string | null {
  const count = statusMessageMediaItems(message, mediaItems).length
  if (!count) return null
  return count === 1 ? t('1 media item') : `${count} ${t('media items')}`
}

export function messagePreview(
  message: { text?: string; body_text_preview?: string | null },
  t: Translate
): string {
  return message.text || message.body_text_preview || t('No preview')
}

export function messageMetaFlags(message: WhatsAppPanelMessage, t: Translate): string[] {
  const metadata = messageMetadata(message)
  const flags: string[] = []
  if (typeof metadata.mention_count === 'number' && metadata.mention_count > 0) {
    flags.push(`@${metadata.mention_count}`)
  }
  if (metadata.whatsapp_view_once) flags.push(t('View once'))
  if (metadata.whatsapp_ephemeral) flags.push(t('Ephemeral'))
  if (metadata.whatsapp_sticker) flags.push(t('Sticker'))
  if (metadata.whatsapp_poll) flags.push(t('Poll'))
  if (metadata.whatsapp_location) flags.push(t('Location'))
  if (metadata.whatsapp_contact_card) flags.push(t('Contact card'))
  if (metadata.whatsapp_system_message) flags.push(t('System'))
  if (metadata.whatsapp_join_leave) flags.push(t('Membership'))
  if (metadata.whatsapp_link_preview) flags.push(t('Link'))
  if (metadata.communication_object_type === 'status') flags.push(t('Status'))
  return flags
}

export function isStatusMessage(message: WhatsAppPanelMessage): boolean {
  const metadata = messageMetadata(message)
  return (
    metadata.communication_object_type === 'status' ||
    message.provider_chat_id === 'status-feed'
  )
}

export function messageMentionNames(message: WhatsAppPanelMessage): string[] {
  return metadataArray(messageMetadata(message).mention_usernames)
    .filter((value): value is string => typeof value === 'string' && value.trim().length > 0)
    .slice(0, 5)
}

export function messageLinkPreview(message: WhatsAppPanelMessage): { title: string | null; url: string | null; site: string | null } | null {
  const preview = metadataRecord(messageMetadata(message).whatsapp_link_preview)
  if (!preview) return null
  return {
    title: metadataString(preview.title),
    url: metadataString(preview.url),
    site: metadataString(preview.site_name ?? preview.site),
  }
}

export function messagePollSummary(message: WhatsAppPanelMessage, t: Translate): string | null {
  const poll = metadataRecord(messageMetadata(message).whatsapp_poll)
  if (!poll) return null
  const title = metadataString(poll.question ?? poll.title)
  const options = metadataArray(poll.options).length
  if (title && options) return `${title} · ${options} ${t('options')}`
  if (title) return title
  if (options) return `${options} ${t('options')}`
  return t('Poll attached')
}

export function messageLocationSummary(message: WhatsAppPanelMessage, t: Translate): string | null {
  const location = metadataRecord(messageMetadata(message).whatsapp_location)
  if (!location) return null
  const label = metadataString(location.label ?? location.name ?? location.address)
  const lat = typeof location.latitude === 'number' ? location.latitude : null
  const lon = typeof location.longitude === 'number' ? location.longitude : null
  if (label && lat != null && lon != null) return `${label} · ${lat}, ${lon}`
  if (label) return label
  if (lat != null && lon != null) return `${lat}, ${lon}`
  return t('Shared location')
}

export function messageContactCardSummary(message: WhatsAppPanelMessage, t: Translate): string | null {
  const card = metadataRecord(messageMetadata(message).whatsapp_contact_card)
  if (!card) return null
  const displayName = metadataString(card.display_name ?? card.name)
  const phones = metadataArray(card.phones)
    .map((entry) => metadataString(metadataRecord(entry)?.value ?? entry))
    .filter((value): value is string => Boolean(value))
  if (displayName && phones[0]) return `${displayName} · ${phones[0]}`
  return displayName ?? phones[0] ?? t('Contact card attached')
}

export function messageStickerSummary(message: WhatsAppPanelMessage, t: Translate): string | null {
  const sticker = metadataRecord(messageMetadata(message).whatsapp_sticker)
  if (!sticker) return null
  return metadataString(sticker.emoji ?? sticker.label ?? sticker.pack_name) ?? t('Sticker attached')
}

export function messageSystemSummary(message: WhatsAppPanelMessage, t: Translate): string | null {
  const metadata = messageMetadata(message)
  const systemMessage = metadataRecord(metadata.whatsapp_system_message)
  if (systemMessage) {
    return metadataString(systemMessage.text ?? systemMessage.kind ?? systemMessage.type) ?? t('System message')
  }
  const joinLeave = metadataRecord(metadata.whatsapp_join_leave)
  if (joinLeave) {
    return metadataString(joinLeave.text ?? joinLeave.action ?? joinLeave.kind) ?? t('Membership update')
  }
  return null
}

export function reactionSummary(message: WhatsAppPanelMessage): TelegramReactionGroup[] {
  const summary = messageMetadata(message).reaction_summary
  if (!summary || typeof summary !== 'object' || !Array.isArray((summary as { reactions?: unknown[] }).reactions)) {
    return []
  }
  return (summary as { reactions: unknown[] }).reactions
    .filter(
      (item): item is { reaction?: unknown; reaction_emoji?: unknown; count?: unknown } =>
        typeof item === 'object' && item !== null
    )
    .map((item) => ({
      reaction_emoji:
        typeof item.reaction_emoji === 'string'
          ? item.reaction_emoji
          : typeof item.reaction === 'string'
            ? item.reaction
            : '',
      count: typeof item.count === 'number' ? item.count : 1,
      senders: [],
    }))
    .filter((item) => item.reaction_emoji)
}

export function useWhatsappCommunicationsPresentation(t: Translate) {
  return {
    isPreviewableMediaItem,
    isStatusMessage,
    mediaLabel,
    mediaMetaLabel,
    mediaTime,
    memberLabel,
    messageContactCardSummary: (message: WhatsAppPanelMessage) => messageContactCardSummary(message, t),
    messageLinkPreview,
    messageLocationSummary: (message: WhatsAppPanelMessage) => messageLocationSummary(message, t),
    messageMentionNames,
    messageMetaFlags: (message: WhatsAppPanelMessage) => messageMetaFlags(message, t),
    messagePollSummary: (message: WhatsAppPanelMessage) => messagePollSummary(message, t),
    messagePreview: (message: { text?: string; body_text_preview?: string | null }) => messagePreview(message, t),
    messageStickerSummary: (message: WhatsAppPanelMessage) => messageStickerSummary(message, t),
    messageSystemSummary: (message: WhatsAppPanelMessage) => messageSystemSummary(message, t),
    messageTime,
    reactionPalette: TELEGRAM_REACTION_PALETTE,
    reactionSummary,
    statusAuthorDetail,
    statusAuthorHeadline,
    statusDeletedSummary: (message: WhatsAppPanelMessage) => statusDeletedSummary(message, t),
    statusMediaCountLabel: (message: WhatsAppPanelMessage, mediaItems: WhatsappWebMediaItem[]) =>
      statusMediaCountLabel(message, mediaItems, t),
    statusMessageMediaItems,
    statusViewSummary: (message: WhatsAppPanelMessage) => statusViewSummary(message, t),
  }
}
