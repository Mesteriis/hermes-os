import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramRawEvidencePanel boundary', () => {
  it('loads source evidence through the query layer instead of inline fetch', () => {
    const source = readFileSync(new URL('./TelegramRawEvidencePanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('useTelegramRawMessageEvidenceQuery')
    expect(source).toContain("t('Raw Source Evidence')")
    expect(source).toContain("t('Sanitized payload')")
    expect(source).toContain("t('Provenance')")
    expect(source).not.toContain('fetch(')
  })
})
