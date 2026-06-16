import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

describe('TelegramCallTranscriptPanel boundary', () => {
  it('loads transcript evidence through the query layer instead of inline fetch', () => {
    const source = readFileSync(
      resolve('src/domains/telegram/components/TelegramCallTranscriptPanel.vue'),
      'utf8'
    )

    expect(source).toContain('useTelegramCallTranscriptQuery')
    expect(source).toContain("t('Transcript')")
    expect(source).toContain("t('No transcript projected for this call yet.')")
    expect(source).not.toContain('fetch(')
  })
})
