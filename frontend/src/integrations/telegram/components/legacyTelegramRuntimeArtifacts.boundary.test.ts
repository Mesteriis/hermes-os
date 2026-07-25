import { existsSync, readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const componentsDir = dirname(fileURLToPath(import.meta.url))
const integrationDir = resolve(componentsDir, '..')
const queriesDir = resolve(componentsDir, '../queries')
const storesDir = resolve(componentsDir, '../stores')

const removedTelegramRuntimeFiles = [
  'TelegramCallsPanel.vue',
  'TelegramCallTranscriptPanel.vue',
  'TelegramCommandAuditPanel.vue',
  'TelegramStatusMessages.vue'
]

const removedTelegramCommunicationsFacadeFiles = [
  resolve(integrationDir, 'api/telegramBusiness.test.ts'),
  resolve(integrationDir, 'api/telegramBusiness.ts'),
  resolve(integrationDir, 'queries/telegramBusinessQueryKeys.ts'),
  resolve(integrationDir, 'queries/telegramChatAvatarSync.ts'),
  resolve(integrationDir, 'queries/useTelegramBusinessMutations.ts'),
  resolve(integrationDir, 'queries/useTelegramBusinessQuery.boundary.test.ts'),
  resolve(integrationDir, 'queries/useTelegramBusinessQuery.ts'),
  resolve(integrationDir, 'types/business.ts'),
  resolve(componentsDir, '../../../domains/communications/components/messengers/TelegramConversationInspector.vue'),
  resolve(componentsDir, '../../../domains/communications/components/messengers/TelegramMessageInspector.vue'),
  resolve(componentsDir, '../../../domains/communications/components/messengers/telegramConversationInspectorActions.test.ts'),
  resolve(componentsDir, '../../../domains/communications/components/messengers/telegramConversationInspectorActions.ts'),
  resolve(componentsDir, '../../../domains/communications/components/messengers/telegramConversationInspectorCommandExtras.test.ts'),
  resolve(componentsDir, '../../../domains/communications/components/messengers/telegramMessageInspectorActions.test.ts'),
  resolve(componentsDir, '../../../domains/communications/components/messengers/telegramMessageInspectorActions.ts'),
  resolve(componentsDir, '../../../domains/communications/queries/useTelegramConversationInspectorController.ts'),
  resolve(componentsDir, '../../../domains/communications/queries/useTelegramMessageInspectorController.ts'),
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

  it('removes the orphaned Communications facade and retains only the generated operational client seam', () => {
    for (const path of removedTelegramCommunicationsFacadeFiles) {
      expect(existsSync(path), path).toBe(false)
    }

    const operationalClientSource = readFileSync(
      resolve(integrationDir, 'api/telegramOperationalClient.ts'),
      'utf8',
    )

    expect(operationalClientSource).toContain('TelegramOperationalService')
    expect(operationalClientSource).toContain('createBrowserGatewayConnectTransport')
    expect(operationalClientSource).not.toContain('ApiClient')
    expect(operationalClientSource).not.toContain('/api/v1/communications')

    const platformRealtimeSource = readFileSync(
      resolve(componentsDir, '../../../platform/bootstrap/realtime.ts'),
      'utf8',
    )
    const mediaUploadWorkflowSource = readFileSync(
      resolve(componentsDir, '../../../platform/bootstrap/useTelegramMediaUploadWorkflow.ts'),
      'utf8',
    )
    const runtimeActionSource = readFileSync(
      resolve(componentsDir, '../../../app/queries/useTelegramConversationRuntimeActions.ts'),
      'utf8',
    )

    expect(platformRealtimeSource).not.toContain('domains/communications/queries/realtimeTelegram')
    expect(platformRealtimeSource).not.toMatch(/\[\s*['"]communications['"]\s*,\s*['"]telegram['"]/)
    expect(mediaUploadWorkflowSource).not.toMatch(/\[\s*['"]communications['"]\s*,\s*['"]telegram['"]/)
    expect(runtimeActionSource).not.toMatch(/\[\s*['"]communications['"]\s*,\s*['"]telegram['"]/)
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
