import { htmlToComposePlainText, plainTextToComposeHtml } from '../richComposeHtml'
import type {
  CommunicationAccountOption,
  ComposeFormModel,
  ComposeMode,
} from '../../types/communications'

type Translate = (key: string, params?: Record<string, string | number>) => string

export function composeEditorHtml(form: ComposeFormModel | null | undefined): string {
  if (!form) return '<p></p>'
  return form.bodyHtml?.trim() ? form.bodyHtml : plainTextToComposeHtml(form.body)
}

export function composeBodyHtmlToPlainText(bodyHtml: string): string {
  return htmlToComposePlainText(bodyHtml)
}

export function composeTitle(mode: ComposeMode | undefined, t: Translate): string {
  if (mode === 'reply') return t('Reply')
  if (mode === 'forward') return t('Forward')
  return t('Compose')
}

export function composePanelState(aiOpen: boolean, contextOpen: boolean): string {
  const panels: string[] = []
  if (aiOpen) panels.push('ai')
  if (contextOpen) panels.push('context')
  return panels.join(' ') || 'none'
}

export function composeFormHasTypedContent(form: ComposeFormModel | null | undefined): boolean {
  if (!form) return false
  const bodyText = htmlToComposePlainText(form.bodyHtml ?? form.body)
  return [
    form.toText,
    form.ccText,
    form.bccText,
    form.subject,
    form.body,
    bodyText,
    form.attachments.length > 0 ? 'attachment' : '',
  ].some((value) => value.trim().length > 0)
}

export function formatComposeAttachmentSize(sizeBytes: number): string {
  if (sizeBytes < 1024) return `${sizeBytes} B`
  if (sizeBytes < 1024 * 1024) return `${Math.ceil(sizeBytes / 1024)} KiB`
  return `${(sizeBytes / (1024 * 1024)).toFixed(1)} MiB`
}

export function composeAccountOptionLabel(
  account: CommunicationAccountOption,
  t: Translate
): string {
  const label = account.email && account.email !== account.label
    ? `${account.label} · ${account.email}`
    : account.label
  return account.can_send ? label : `${label} · ${t('Read only')}`
}
