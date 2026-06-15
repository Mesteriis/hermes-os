import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MessageAiReplyPanel boundary', () => {
  it('renders AI reply review controls without direct API access', () => {
    const source = readFileSync(new URL('./MessageAiReplyPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('AI Reply Review')
    expect(source).toContain('selectedAiReplyTone')
    expect(source).toContain('selectedAiReplyLanguage')
    expect(source).toContain('generateAiReply')
    expect(source).toContain('useGenerateAiReplyVariantsMutation')
    expect(source).toContain('generateVariants')
    expect(source).toContain('replyVariants')
    expect(source).toContain('applyAiReply')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
  })
})
