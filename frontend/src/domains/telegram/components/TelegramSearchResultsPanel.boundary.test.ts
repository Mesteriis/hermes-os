import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramSearchResultsPanel media boundary', () => {
  it('allows projected media search hits to reopen owning message context without inline fetch', () => {
    const source = readFileSync(new URL('./TelegramSearchResultsPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('openMedia: [item: TelegramMediaItem]')
    expect(source).toContain("@click=\"emit('openMedia', item)\"")
    expect(source).toContain('telegramMediaReadiness')
    expect(source).toContain('props.chats.length + props.total + props.mediaItems.length')
    expect(source).toContain('sourceLabel: string')
    expect(source).toContain('mediaSourceLabel: string')
    expect(source).toContain('telegram-search-results__source')
    expect(source).toContain('telegram-search-results__media-source')
    expect(source).toContain('v-if="mediaSourceLabel"')
    expect(source).toContain("t('Media In Current Chat')")
    expect(source).not.toContain('fetch(')
  })
})
