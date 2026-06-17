import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramMessageReferencePanel boundary', () => {
  it('renders projected lifecycle and reference evidence instead of raw provider ids only', () => {
    const source = readFileSync(new URL('./TelegramMessageReferencePanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('target_message_summary')
    expect(source).toContain('source_message_summary')
    expect(source).toContain('useTelegramMessageVersionsQuery')
    expect(source).toContain('useTelegramMessageTombstonesQuery')
    expect(source).toContain('useTelegramMessageReactionsQuery')
    expect(source).toContain('useTelegramCommandsQuery')
    expect(source).toContain('TelegramMessageSourceEvidencePanel')
    expect(source).toContain('hasTelegramSourceEvidence')
    expect(source).toContain('matchesTelegramSourceEvidence')
    expect(source).toContain('currentMessage: TelegramMessage')
    expect(source).toContain("'openMessage',")
    expect(source).toContain('buildFocusedMessage(')
    expect(source).toContain('replyTargetBody')
    expect(source).toContain('replyBody')
    expect(source).toContain('forwardTitle')
    expect(source).toContain('referenceSearchQuery')
    expect(source).toContain("t('Filter references, lifecycle and commands')")
    expect(source).toContain("t('No reference evidence matches this filter.')")
    expect(source).toContain("t('Edit History')")
    expect(source).toContain("t('Tombstones')")
    expect(source).toContain("t('Recent Commands')")
    expect(source).toContain('summarizeTelegramVersionDelta')
    expect(source).toContain('summarizeTelegramTombstoneState')
    expect(source).toContain('summarizeTelegramCommandEvidence')
    expect(source).not.toContain('fetch(')
  })
})
