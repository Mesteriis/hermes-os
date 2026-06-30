import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('ThreadConversationView boundary', () => {
  it('preserves thread expansion, body parsing, translation, and reply orchestration in TS after removing the thread render layer', () => {
    const surfaceSource = readFileSync(
      new URL('../queries/useCommunicationsPageSurface.ts', import.meta.url),
      'utf8'
    )
    const actionQuerySource = readFileSync(
      new URL('../queries/mailActionQueries.ts', import.meta.url),
      'utf8'
    )
    const presentationSource = readFileSync(
      new URL('./threadConversationPresentation.ts', import.meta.url),
      'utf8'
    )
    const bodySource = readFileSync(new URL('./threadMessageBody.ts', import.meta.url), 'utf8')
    const attachmentSource = readFileSync(new URL('./attachmentTable.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./ThreadConversationView.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./ThreadInlineReplyComposer.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./ThreadAttachmentInsightPanel.vue', import.meta.url))).toBe(false)
    expect(surfaceSource).toContain('handleReplyToThreadMessage')
    expect(surfaceSource).toContain('handleSaveThreadReplyDraft')
    expect(surfaceSource).toContain('handleSendThreadReply')
    expect(surfaceSource).toContain('handleOpenThreadMessage')
    expect(surfaceSource).toContain('isThreadReplySending')
    expect(actionQuerySource).toContain('export function useTranslateThreadMutation()')
    expect(presentationSource).toContain('defaultExpandedThreadMessageIds')
    expect(presentationSource).toContain('hasQuotedThreadMessages')
    expect(presentationSource).toContain('summarizeThreadExpansion')
    expect(bodySource).toContain('splitThreadMessageBody')
    expect(bodySource).toContain('previewThreadMessageBody')
    expect(attachmentSource).toContain('formatAttachmentSize')
    expect(attachmentSource).toContain('scanStatusClass')
  })
})
