import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramMessageSourceEvidencePanel boundary', () => {
  it('renders Telegram source metadata evidence without inline fetch or write controls', () => {
    const source = readFileSync(new URL('./TelegramMessageSourceEvidencePanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('TelegramRawEvidencePanel')
    expect(source).toContain('buildTelegramMessageLinkEvidence')
    expect(source).toContain('buildTelegramStructuredEvidence')
    expect(source).toContain('buildTelegramCustomReactionEvidence')
    expect(source).toContain("t('Message Link')")
    expect(source).toContain("t('Open provider permalink')")
    expect(source).toContain("t('Structured Evidence')")
    expect(source).toContain("t('Custom Telegram reaction evidence')")
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('addTelegramReaction')
    expect(source).not.toContain('removeTelegramReaction')
  })
})
