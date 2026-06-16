import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

describe('TelegramRail runtime diagnostics boundary', () => {
  it('renders TDLib diagnostics plus projected member search, calls panel and command audit surfaces', () => {
    const source = readFileSync(
      resolve('src/domains/telegram/components/TelegramRail.vue'),
      'utf8'
    )

    expect(source).toContain('tdjson_path')
    expect(source).toContain('tdjson_runtime_available')
    expect(source).toContain('telegram_api_id_configured')
    expect(source).toContain('telegram_api_hash_configured')
    expect(source).toContain('runtime_blockers')
    expect(source).toContain('tdjson_probe_error')
    expect(source).toContain('TelegramAccountManager')
    expect(source).toContain('TelegramCommandAuditPanel')
    expect(source).toContain('TelegramCallsPanel')
    expect(source).toContain('filteredChatMembers')
    expect(source).toContain("t('Search projected members')")
    expect(source).toContain("t('No projected members match this search.')")
    expect(source).not.toContain('fetch(')
  })
})
