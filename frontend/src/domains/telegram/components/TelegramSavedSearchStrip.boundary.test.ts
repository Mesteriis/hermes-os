import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramSavedSearchStrip boundary', () => {
  it('reuses shared Communication saved searches with Telegram channel scope and no inline API calls', () => {
    const source = readFileSync(new URL('./TelegramSavedSearchStrip.vue', import.meta.url), 'utf8')

    expect(source).toContain("import SavedSearchStrip from '../../communications/components/SavedSearchStrip.vue'")
    expect(source).toContain('useTelegramStore')
    expect(source).toContain('store.telegramSearchQuery = savedSearch.query')
    expect(source).toContain('current-channel-kind="telegram"')
    expect(source).toContain('current-local-state="active"')
    expect(source).toContain('@select="selectSavedSearch"')
    expect(source).toContain('@deleted="clearDeletedSavedSearch"')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
  })
})
