import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('MessageLocalIntelligencePanel boundary', () => {
  it('preserves explain and language-detection mutation contracts after removing the local intelligence render layer', () => {
    const actionQuerySource = readFileSync(
      new URL('../queries/mailActionQueries.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./MessageLocalIntelligencePanel.vue', import.meta.url))).toBe(false)
    expect(actionQuerySource).toContain('export function useExplainMessageMutation()')
    expect(actionQuerySource).toContain('export function useDetectMessageLanguageMutation()')
  })

})
