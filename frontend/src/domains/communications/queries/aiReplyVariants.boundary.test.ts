import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('AI reply variants query boundary', () => {
  it('wraps reply variants behind a TanStack mutation', () => {
    const source = readFileSync(new URL('./mailActionQueries.ts', import.meta.url), 'utf8')

    expect(source).toContain('generateAiReplyVariants')
    expect(source).toContain('useGenerateAiReplyVariantsMutation')
    expect(source).toContain('AiReplyVariantsResponse')
    expect(source).toContain('languages')
    expect(source).toContain('tones')
    expect(source).not.toContain('fetch(')
  })
})
