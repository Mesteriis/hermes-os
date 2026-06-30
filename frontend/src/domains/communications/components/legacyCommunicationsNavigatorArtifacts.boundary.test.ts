import { existsSync, readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const componentDir = dirname(fileURLToPath(import.meta.url))
const queriesDir = resolve(componentDir, '../queries')
const viewsDir = resolve(componentDir, '../views')

const removedNavigatorFiles = [
  'BulkActionsBar.vue',
  'CommunicationList.vue',
  'CommunicationListItem.vue',
  'CommunicationsActionBar.vue',
  'CommunicationsCallsPanel.vue',
  'CommunicationsContextInspector.vue',
  'CommunicationsContextRail.vue',
  'CommunicationsConversationList.vue',
  'CommunicationsListPane.vue',
  'CommunicationsRailPane.vue',
  'CommunicationsTopbarSlot.vue',
  'CommunicationsWorkbench.vue',
  'DraftStrip.vue',
  'HealthStrip.vue',
  'MailCertificateStrip.vue',
  'MailResourceOverviewStrip.vue',
  'OutboxStatusStrip.vue'
]

function readComponentArtifact(relativePath: string): string {
  return readFileSync(resolve(componentDir, relativePath), 'utf8')
}

function readQueryArtifact(relativePath: string): string {
  return readFileSync(resolve(queriesDir, relativePath), 'utf8')
}

function readViewArtifact(relativePath: string): string {
  return readFileSync(resolve(viewsDir, relativePath), 'utf8')
}

describe('legacy communications navigator artifacts', () => {
  it('removes the remaining communications navigation and inspector Vue layer', () => {
    for (const relativePath of removedNavigatorFiles) {
      expect(existsSync(resolve(componentDir, relativePath))).toBe(false)
    }
  })

  it('keeps communications page orchestration in TypeScript surfaces and queries', () => {
    const pageSurfaceSource = readQueryArtifact('useCommunicationsPageSurface.ts')
    const outboxStripSource = readQueryArtifact('outboxStatusStrip.ts')
    const queryBarrelSource = readQueryArtifact('useCommunicationsQuery.ts')
    const resourceOverviewSource = readViewArtifact('useMailResourceOverview.ts')

    expect(pageSurfaceSource).toContain('useMailListQuery')
    expect(pageSurfaceSource).toContain('useOutboxStatusStrip')
    expect(pageSurfaceSource).toContain('useMailResourceOverview')
    expect(pageSurfaceSource).toContain('useMailSyncActions')
    expect(pageSurfaceSource).toContain('useThreadReplyActions')
    expect(pageSurfaceSource).toContain('useSelectedMessageActions')
    expect(pageSurfaceSource).toContain('draftToComposeForm')
    expect(outboxStripSource).toContain('useOutboxQuery')
    expect(outboxStripSource).toContain('useUndoOutboxMutation')
    expect(outboxStripSource).toContain('undoOutbox')
    expect(outboxStripSource).toContain('prefetchMoreOutboxItems')
    expect(queryBarrelSource).toContain("export * from './mailActionQueries'")
    expect(queryBarrelSource).toContain("export * from './mailCoreQueries'")
    expect(queryBarrelSource).toContain("export * from './mailOperationQueries'")
    expect(queryBarrelSource).toContain("export * from './mailWorkspaceQueries'")
    expect(resourceOverviewSource).toContain('useSubscriptionsQuery')
    expect(resourceOverviewSource).toContain('useTopSendersQuery')
    expect(resourceOverviewSource).toContain('useCommunicationBlockersQuery')
  })

  it('preserves non-visual outbox logic in standalone TypeScript helpers', () => {
    const outboxPresentationSource = readComponentArtifact('outboxStatus.ts')

    expect(outboxPresentationSource).toContain('export type OutboxStatusPresentation')
    expect(outboxPresentationSource).toContain('outboxStatusPresentation')
    expect(outboxPresentationSource).toContain('visibleOutboxStatusItems')
    expect(outboxPresentationSource).toContain("title: 'Delivery failed'")
    expect(outboxPresentationSource).toContain("title: 'Undo available'")
    expect(outboxPresentationSource).toContain("title: 'Scheduled'")
  })
})
