import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('MessageAiReplyPanel boundary', () => {
  it('preserves AI reply mutation contracts after removing the AI reply render layer', () => {
    const actionQuerySource = readFileSync(
      new URL('../queries/mailActionQueries.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./MessageAiReplyPanel.vue', import.meta.url))).toBe(false)
    expect(actionQuerySource).toContain('export function useGenerateAiReplyVariantsMutation()')
  })
})
