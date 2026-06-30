import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('CommunicationsDetailPane boundary', () => {
  it('preserves message and thread selection orchestration after removing the detail pane render layer', () => {
    const surfaceSource = readFileSync(
      new URL('../queries/useCommunicationsPageSurface.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./CommunicationsDetailPane.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./CommunicationViewer.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./ThreadConversationView.vue', import.meta.url))).toBe(false)
    expect(surfaceSource).toContain('selectedThreadMessages')
    expect(surfaceSource).toContain('selectedThreadErrorMessage')
    expect(surfaceSource).toContain('handleSelectThread')
    expect(surfaceSource).toContain('handleOpenThreadMessage')
    expect(surfaceSource).toContain('handleReplyToThreadMessage')
    expect(surfaceSource).toContain('handleSaveThreadReplyDraft')
    expect(surfaceSource).toContain('handleSendThreadReply')
    expect(surfaceSource).toContain('isThreadReplySending')
    expect(surfaceSource).toContain('handleDeleteFromProvider')
    expect(surfaceSource).toContain('handleMarkMessageRead')
    expect(surfaceSource).toContain('handleMarkMessageUnread')
    expect(surfaceSource).toContain('handleForwardMessage')
    expect(surfaceSource).toContain('handleRedirectMessage')
    expect(surfaceSource).toContain('handleReplyAll')
  })
})
