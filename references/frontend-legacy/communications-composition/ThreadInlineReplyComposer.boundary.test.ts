// Historical pre-clean-room orchestration test. Not part of the active validation suite.
import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('ThreadInlineReplyComposer boundary', () => {
  it('preserves thread reply orchestration after removing the inline reply composer render layer', () => {
    const surfaceSource = readFileSync(
      new URL('../queries/useCommunicationsPageSurface.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./ThreadInlineReplyComposer.vue', import.meta.url))).toBe(false)
    expect(surfaceSource).toContain('handleReplyToThreadMessage')
    expect(surfaceSource).toContain('handleSaveThreadReplyDraft')
    expect(surfaceSource).toContain('handleSendThreadReply')
    expect(surfaceSource).toContain('isThreadReplySending')
  })
})
