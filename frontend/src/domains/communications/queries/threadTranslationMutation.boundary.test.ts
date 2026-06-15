import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('thread translation mutation boundary', () => {
  it('routes thread translation through TanStack mutation and the communications API client', () => {
    const source = readFileSync(new URL('./mailActionQueries.ts', import.meta.url), 'utf8')

    expect(source).toContain('translateThread')
    expect(source).toContain('ThreadTranslationResponse')
    expect(source).toContain('export function useTranslateThreadMutation()')
    expect(source).toContain('useMutation<')
    expect(source).toContain('translateThread(accountId, subject, targetLanguage, limit)')
  })
})
