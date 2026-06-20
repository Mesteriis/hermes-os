import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

describe('TelegramReadProgressPanel boundary', () => {
  it('renders provider read progress and recent read commands through query/state helpers', () => {
    const source = readFileSync(
      resolve('src/integrations/telegram/components/TelegramReadProgressPanel.vue'),
      'utf8'
    )

    expect(source).toContain('useTelegramCommandsQuery')
    expect(source).toContain("providerChatId: computed(() => props.selectedChat?.provider_chat_id ?? null)")
    expect(source).toContain("commandKinds: computed(() => ['mark_read', 'mark_unread'])")
    expect(source).toContain('telegramThreadReadProgress')
    expect(source).toContain('telegramLatestReadableProviderMessageId')
    expect(source).toContain('telegramCommandAuditState')
    expect(source).toContain('telegramCommandSubject')
    expect(source).toContain('command.command_kind === \'mark_read\' || command.command_kind === \'mark_unread\'')
    expect(source).toContain("t('Read Progress')")
    expect(source).toContain("t('Provider boundary')")
    expect(source).toContain("t('Loaded boundary')")
    expect(source).toContain("t('Latest visible message')")
    expect(source).toContain("t('Projection is fully read')")
    expect(source).toContain('telegram-read-progress__commands')
    expect(source).not.toContain('fetch(')
  })
})
