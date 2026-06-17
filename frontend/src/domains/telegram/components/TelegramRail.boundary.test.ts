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
    expect(source).toContain('tdlib_chat_type')
    expect(source).toContain('tdlib_supergroup_id')
    expect(source).toContain('is_supergroup')
    expect(source).toContain('is_channel_supergroup')
    expect(source).toContain('is_forum')
    expect(source).toContain('tdlib_permissions')
    expect(source).toContain('permissionsSummary')
    expect(source).toContain("t('Permissions')")
    expect(source).toContain('syncTarget')
    expect(source).toContain('commandTarget')
    expect(source).toContain('TelegramAccountManager')
    expect(source).toContain('TelegramCommandAuditPanel')
    expect(source).toContain('TelegramCallsPanel')
    expect(source).toContain('TelegramMembersPanel')
    expect(source).toContain(':chatMembers="chatMembers"')
    expect(source).toContain(':capabilities="capabilities"')
    expect(source).toContain(':accountId="selectedTelegramChat?.account_id ?? null"')
    expect(source).toContain(':providerChatId="selectedTelegramChat?.provider_chat_id ?? null"')
    expect(source).not.toContain('fetch(')
  })
})
