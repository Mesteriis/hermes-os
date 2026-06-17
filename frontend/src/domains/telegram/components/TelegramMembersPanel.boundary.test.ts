import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

describe('TelegramMembersPanel boundary', () => {
  it('renders provider roster state through TanStack mutation hooks without direct fetch', () => {
    const source = readFileSync(
      resolve('src/domains/telegram/components/TelegramMembersPanel.vue'),
      'utf8'
    )

    expect(source).toContain('useTelegramChatMembersQuery')
    expect(source).toContain('useSyncTelegramChatMembersMutation')
    expect(source).toContain('useJoinTelegramChatMutation')
    expect(source).toContain('useLeaveTelegramChatMutation')
    expect(source).toContain('useTelegramCommandsQuery')
    expect(source).toContain("commandKinds: () => ['join', 'leave']")
    expect(source).toContain('useTelegramCommandRetryMutation')
    expect(source).toContain('telegramParticipantLifecycleCommands')
    expect(source).toContain('telegramCommandAuditState')
    expect(source).toContain("t('Search provider members')")
    expect(source).toContain("t('Role filter')")
    expect(source).toContain("t('Sync provider roster')")
    expect(source).toContain("t('Load more members')")
    expect(source).toContain("capabilityEnabled('participants.join')")
    expect(source).toContain("capabilityEnabled('participants.leave')")
    expect(source).toContain("t('Join chat')")
    expect(source).toContain("t('Leave chat')")
    expect(source).toContain("t('Retry command')")
    expect(source).toContain("member.source === 'message_heuristic'")
    expect(source).toContain('member.is_owner')
    expect(source).toContain('member.is_admin')
    expect(source).toContain('membersQuery.fetchNextPage()')
    expect(source).toContain('permissionSummary')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
  })
})
