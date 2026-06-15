import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('ThreadInlineReplyComposer boundary', () => {
  it('owns inline reply editing and send review without API or cache logic', () => {
    const source = readFileSync(
      new URL('./ThreadInlineReplyComposer.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('message: ThreadMessage')
    expect(source).toContain('bodyHtml: string')
    expect(source).toContain('isSendingReply: boolean')
    expect(source).toContain('RichComposeEditor')
    expect(source).toContain('reviewingReply')
    expect(source).toContain('update:bodyHtml')
    expect(source).toContain('saveDraft: []')
    expect(source).toContain('continueInCompose: []')
    expect(source).toContain('send: []')
    expect(source).toContain('Review reply before sending')
    expect(source).toContain('Immediate provider send')
    expect(source).toContain("{{ isSendingReply ? 'Sending...' : 'Send' }}")
    expect(source).toContain('replyReviewRecipient')
    expect(source).toContain('replyReviewSubject')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
    expect(source).not.toContain('useQuery')
  })
})
