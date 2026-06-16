import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

describe('TelegramCallsPanel boundary', () => {
  it('loads projected call metadata through the query layer and filters locally before transcript rendering', () => {
    const source = readFileSync(
      resolve('src/domains/telegram/components/TelegramCallsPanel.vue'),
      'utf8'
    )

    expect(source).toContain('useTelegramCallsQuery')
    expect(source).toContain('filteredCalls')
    expect(source).toContain('TelegramCallTranscriptPanel')
    expect(source).toContain("t('Search projected calls')")
    expect(source).toContain("t('Recent Calls')")
    expect(source).not.toContain('fetch(')
  })
})
