import type { RichTextEditorToolbarAction, UtilityTone } from '@/shared/ui'
import type { MessengerConversationModel, MessengerConversationKind } from './messengerElements'

export type MessengerComposerCapability = {
  id: string
  label: string
  icon: string
  tone?: UtilityTone
}

export type MessengerComposerPreset = {
  id: string
  providerLabel: string
  variantLabel: string
  icon: string
  helper: string
  placeholder: string
  toolbarLabel: string
  maxLength: number
  primaryActions: readonly MessengerComposerCapability[]
  insertActions: readonly MessengerComposerCapability[]
  richTextActions: readonly RichTextEditorToolbarAction[]
}

const basicMessengerRichTextActions: readonly RichTextEditorToolbarAction[] = [
  { id: 'bold', label: 'Bold', icon: 'tabler:bold', group: 'marks' },
  { id: 'italic', label: 'Italic', icon: 'tabler:italic', group: 'marks' },
  { id: 'strike', label: 'Strike', icon: 'tabler:strikethrough', group: 'marks' },
  { id: 'link', label: 'Link evidence', icon: 'tabler:link', group: 'insert' },
  { id: 'clearFormatting', label: 'Clear formatting', icon: 'tabler:eraser', group: 'cleanup' }
]

const telegramRichTextActions: readonly RichTextEditorToolbarAction[] = [
  { id: 'quote', label: 'Quote', icon: 'tabler:quote', group: 'structure' },
  ...basicMessengerRichTextActions,
  { id: 'code', label: 'Inline code', icon: 'tabler:code', group: 'marks' }
]

const whatsAppRichTextActions: readonly RichTextEditorToolbarAction[] = [
  { id: 'quote', label: 'Quote', icon: 'tabler:quote', group: 'structure' },
  ...basicMessengerRichTextActions,
  { id: 'code', label: 'Inline code', icon: 'tabler:code', group: 'marks' }
]

const signalRichTextActions: readonly RichTextEditorToolbarAction[] = [
  { id: 'quote', label: 'Quote', icon: 'tabler:quote', group: 'structure' },
  { id: 'bold', label: 'Bold', icon: 'tabler:bold', group: 'marks' },
  { id: 'italic', label: 'Italic', icon: 'tabler:italic', group: 'marks' },
  { id: 'link', label: 'Link evidence', icon: 'tabler:link', group: 'insert' },
  { id: 'clearFormatting', label: 'Clear formatting', icon: 'tabler:eraser', group: 'cleanup' }
]

export function telegramMessengerComposerPreset(conversation: MessengerConversationModel): MessengerComposerPreset {
  return {
    id: 'telegram',
    providerLabel: 'Telegram',
    variantLabel: messengerComposerVariantLabel(conversation.kind),
    icon: 'tabler:brand-telegram',
    helper: 'Telegram composer helper',
    placeholder: 'Write a Telegram reply',
    toolbarLabel: 'Telegram rich text tools',
    maxLength: 4096,
    primaryActions: [
      { id: 'telegram-file', label: 'Attach file', icon: 'tabler:paperclip', tone: 'accent' }
    ],
    // Voice, polls, scheduling and hidden-send options are not provider capabilities yet.
    insertActions: [],
    richTextActions: telegramRichTextActions
  }
}

export function whatsAppMessengerComposerPreset(conversation: MessengerConversationModel): MessengerComposerPreset {
  const primaryActions: MessengerComposerCapability[] = [
    { id: 'whatsapp-reply', label: 'Quote reply', icon: 'tabler:corner-up-left', tone: 'accent' },
    { id: 'whatsapp-template', label: 'Template reply', icon: 'tabler:template', tone: 'success' },
    { id: 'whatsapp-emoji', label: 'Emoji', icon: 'tabler:mood-smile', tone: 'warning' }
  ]

  if (conversation.kind === 'group') {
    primaryActions.push({ id: 'whatsapp-mention', label: 'Mention participant', icon: 'tabler:at', tone: 'accent' })
  }

  return {
    id: 'whatsapp',
    providerLabel: 'WhatsApp',
    variantLabel: messengerComposerVariantLabel(conversation.kind),
    icon: 'tabler:brand-whatsapp',
    helper: 'WhatsApp composer helper',
    placeholder: 'Write a WhatsApp reply',
    toolbarLabel: 'WhatsApp rich text tools',
    maxLength: 4096,
    primaryActions,
    insertActions: [
      { id: 'whatsapp-media', label: 'Attach media', icon: 'tabler:photo-plus', tone: 'accent' },
      { id: 'whatsapp-voice', label: 'Voice note', icon: 'tabler:microphone', tone: 'info' },
      { id: 'whatsapp-contact', label: 'Contact card', icon: 'tabler:user-square', tone: 'neutral' },
      { id: 'whatsapp-location', label: 'Share location', icon: 'tabler:map-pin', tone: 'warning' }
    ],
    richTextActions: whatsAppRichTextActions
  }
}

export function signalMessengerComposerPreset(conversation: MessengerConversationModel): MessengerComposerPreset {
  return {
    id: 'signal',
    providerLabel: 'Signal',
    variantLabel: messengerComposerVariantLabel(conversation.kind),
    icon: 'tabler:message-lock',
    helper: 'Signal composer helper',
    placeholder: 'Write a Signal reply',
    toolbarLabel: 'Signal rich text tools',
    maxLength: 4096,
    primaryActions: [
      { id: 'signal-reply', label: 'Quote reply', icon: 'tabler:corner-up-left', tone: 'accent' },
      { id: 'signal-disappearing', label: 'Disappearing message', icon: 'tabler:hourglass', tone: 'warning' },
      { id: 'signal-safety', label: 'Safety number', icon: 'tabler:shield-check', tone: 'success' }
    ],
    insertActions: [
      { id: 'signal-secure-attachment', label: 'Secure attachment', icon: 'tabler:paperclip', tone: 'accent' },
      { id: 'signal-voice', label: 'Voice note', icon: 'tabler:microphone', tone: 'info' }
    ],
    richTextActions: signalRichTextActions
  }
}

export function messengerComposerDraftHtml(draft: string): string {
  const escapedDraft = escapeHtml(draft.trim())
  if (!escapedDraft) return '<p></p>'
  return `<p>${escapedDraft.replace(/\n{2,}/g, '</p><p>').replace(/\n/g, '<br>')}</p>`
}

export function messengerComposerPlainText(html: string): string {
  return html
    .replace(/<\/(p|div|li|h[1-6])>/gi, '\n')
    .replace(/<br\s*\/?>/gi, '\n')
    .replace(/<[^>]+>/g, '')
    .replace(/&nbsp;/g, ' ')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&amp;/g, '&')
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)
    .join('\n')
    .trim()
}

export function messengerComposerCapabilityCanOpenFile(
  capability: Pick<MessengerComposerCapability, 'id'>,
  isActionRunning: boolean
): boolean {
  return capability.id === 'telegram-file' && !isActionRunning
}

export function localizedMessengerRichTextActions(
  actions: readonly RichTextEditorToolbarAction[],
  translate: (key: string) => string
): RichTextEditorToolbarAction[] {
  const localizedActions: RichTextEditorToolbarAction[] = []

  for (const action of actions) {
    localizedActions.push({
      ...action,
      label: translate(action.label)
    })
  }

  return localizedActions
}

function messengerComposerVariantLabel(kind: MessengerConversationKind): string {
  if (kind === 'direct') return 'Direct composer'
  if (kind === 'group') return 'Group composer'
  return 'Channel composer'
}

function escapeHtml(value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;')
}
