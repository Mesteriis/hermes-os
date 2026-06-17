import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

describe('TelegramCommandAuditPanel boundary', () => {
  it('loads durable command rows through the query layer and filters them locally', () => {
    const source = readFileSync(
      resolve('src/domains/telegram/components/TelegramCommandAuditPanel.vue'),
      'utf8'
    )

    expect(source).toContain('useTelegramCommandsQuery')
    expect(source).toContain('telegramCommandAuditState')
    expect(source).toContain('telegramCommandRetrySummary')
    expect(source).toContain('filteredCommands')
    expect(source).toContain("t('Current chat only')")
    expect(source).toContain("t('Search command rows')")
    expect(source).toContain("t('Recent Commands')")
    expect(source).toContain('telegram-command-audit__item--dead-letter')
    expect(source).not.toContain('fetch(')
  })
})
