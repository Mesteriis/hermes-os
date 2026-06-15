import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('message local intelligence query boundary', () => {
  it('wraps explain and language APIs through TanStack mutations', () => {
    const source = readFileSync(new URL('./mailActionQueries.ts', import.meta.url), 'utf8')

    expect(source).toContain('useExplainMessageMutation')
    expect(source).toContain('useDetectMessageLanguageMutation')
    expect(source).toContain('fetchMessageExplain')
    expect(source).toContain('detectMessageLanguage')
  })

})
