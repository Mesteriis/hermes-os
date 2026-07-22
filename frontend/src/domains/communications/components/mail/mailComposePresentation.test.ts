import { describe, expect, it } from 'vitest'
import type { CommunicationAccountOption, ComposeFormModel } from '../../types/communications'
import {
  composeAccountOptionLabel,
  composeEditorHtml,
  composeFormHasTypedContent,
  composePanelState,
  composeTitle,
  formatComposeAttachmentSize,
} from './mailComposePresentation'

describe('mail compose presentation', () => {
  it('projects editor, title and panel state', () => {
    const form = composeForm({ body: 'Hello', bodyHtml: null })
    const t = (key: string) => key

    expect(composeEditorHtml(form)).toContain('Hello')
    expect(composeTitle('reply', t)).toBe('Reply')
    expect(composePanelState(true, false)).toBe('ai')
    expect(composePanelState(true, true)).toBe('ai context')
  })

  it('detects dirty drafts and formats account/attachment labels', () => {
    const t = (key: string) => key
    const account: CommunicationAccountOption = {
      account_id: 'account-1', label: 'Personal', provider_kind: 'mail', email: 'owner@example.test',
      can_send: false, send_unavailable_reason: 'readonly',
    }

    expect(composeFormHasTypedContent(composeForm({ body: 'draft' }))).toBe(true)
    expect(composeFormHasTypedContent(composeForm())).toBe(false)
    expect(composeAccountOptionLabel(account, t)).toBe('Personal · owner@example.test · Read only')
    expect(formatComposeAttachmentSize(1024 * 1024)).toBe('1.0 MiB')
  })
})

function composeForm(overrides: Partial<ComposeFormModel> = {}): ComposeFormModel {
  return {
    mode: 'compose', draftId: '', accountId: '', toText: '', ccText: '', bccText: '', subject: '',
    body: '', bodyHtml: null, bodyFormat: 'plain', scheduledSendAt: '', undoSendSeconds: null,
    inReplyTo: null, attachments: [], ...overrides,
  }
}
