import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('BilingualReplyPanel boundary', () => {
  it('uses Vee/Zod forms and a TanStack mutation without direct API calls', () => {
    const source = readFileSync(
      new URL('./BilingualReplyPanel.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain("from 'vee-validate'")
    expect(source).toContain('../forms/bilingualReplyFlowForm')
    expect(source).toContain('usePrepareBilingualReplyFlowMutation')
    expect(source).toContain('setFieldValue')
    expect(source).toContain('Original')
    expect(source).toContain('Translation')
    expect(source).toContain('Reply in Russian')
    expect(source).toContain('Back Translation')
    expect(source).toContain('sendBilingualReply')
    expect(source).toContain('@submit.prevent="submitBilingualReply"')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
  })
})
