import { existsSync, readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const componentsDir = dirname(fileURLToPath(import.meta.url))
const queriesDir = resolve(componentsDir, '../queries')
const storesDir = resolve(componentsDir, '../stores')

const removedTelegramRuntimeFiles = [
  'TelegramCallsPanel.vue',
  'TelegramCallTranscriptPanel.vue',
  'TelegramCommandAuditPanel.vue',
  'TelegramStatusMessages.vue'
]

function readQueryArtifact(relativePath: string): string {
  return readFileSync(resolve(queriesDir, relativePath), 'utf8')
}

function readStoreArtifact(relativePath: string): string {
  return readFileSync(resolve(storesDir, relativePath), 'utf8')
}

describe('legacy telegram runtime artifacts', () => {
  it('removes the orphaned Telegram runtime Vue render layer', () => {
    for (const relativePath of removedTelegramRuntimeFiles) {
      expect(existsSync(resolve(componentsDir, relativePath))).toBe(false)
    }
  })

  it('preserves Telegram runtime query contracts in TypeScript composables', () => {
    const runtimeQuerySource = readQueryArtifact('useTelegramQuery.ts')
    const lifecycleQuerySource = readQueryArtifact('useTelegramLifecycleQuery.ts')

    expect(runtimeQuerySource).toContain('useTelegramCallsQuery')
    expect(runtimeQuerySource).toContain('useTelegramCallTranscriptQuery')
    expect(runtimeQuerySource).toContain('fetchTelegramCalls')
    expect(runtimeQuerySource).toContain('fetchTelegramCallTranscript')
    expect(runtimeQuerySource).toContain('computedTelegramCallsQueryKey')
    expect(lifecycleQuerySource).toContain('useTelegramCommandsQuery')
    expect(lifecycleQuerySource).toContain('useTelegramCommandRetryMutation')
    expect(lifecycleQuerySource).toContain('retryTelegramCommand')
    expect(lifecycleQuerySource).toContain("queryKey: ['integrations', 'telegram', 'commands', command.account_id]")
  })

  it('preserves Telegram command audit business logic in stores instead of Vue files', () => {
    const auditStoreSource = readStoreArtifact('telegramCommandAudit.ts')

    expect(auditStoreSource).toContain('export type TelegramCommandAuditState')
    expect(auditStoreSource).toContain('telegramCommandRetrySummary')
    expect(auditStoreSource).toContain('telegramCommandSubject')
    expect(auditStoreSource).toContain('telegramCommandAuditState')
    expect(auditStoreSource).toContain('reactionMismatchDetail')
    expect(auditStoreSource).toContain('chatLifecycleMismatchDetail')
    expect(auditStoreSource).toContain('messageLifecycleDetail')
  })
})
