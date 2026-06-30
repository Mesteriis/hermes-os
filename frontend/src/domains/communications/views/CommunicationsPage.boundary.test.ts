import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('CommunicationsPage boundary', () => {
  it('preserves communications orchestration after removing the CommunicationsPage Vue layer', () => {
    const surfaceSource = readFileSync(
      new URL('../queries/useCommunicationsPageSurface.ts', import.meta.url),
      'utf8'
    )
    const resourceOverviewSource = readFileSync(
      new URL('./useMailResourceOverview.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./CommunicationsPage.vue', import.meta.url))).toBe(false)

    expect(surfaceSource).toContain('useMailListQuery')
    expect(surfaceSource).toContain('useFolderMailList')
    expect(surfaceSource).toContain('useThreadReplyActions')
    expect(surfaceSource).toContain('useMailSyncActions')
    expect(surfaceSource).toContain('useSelectedMessageActions')
    expect(surfaceSource).toContain('handleBulkAction')
    expect(surfaceSource).toContain('useMailResourceOverview')
    expect(surfaceSource).toContain('useOutboxStatusStrip')
    expect(surfaceSource).toContain('savedSearchChannelKind = ref<string>()')
    expect(surfaceSource).toContain('handleGenerateAiReply')
    expect(surfaceSource).toContain('handleApplyAiReply')
    expect(surfaceSource).toContain('handleReviewSecurity')
    expect(surfaceSource).toContain('handleReviewRecipients')
    expect(surfaceSource).toContain('handleReplyAll')
    expect(surfaceSource).toContain('handleForwardMessage')
    expect(surfaceSource).toContain('handleRedirectMessage')
    expect(surfaceSource).toContain('handleMarkMessageRead')
    expect(surfaceSource).toContain('handleMarkMessageUnread')
    expect(surfaceSource).toContain('handleDeleteFromProvider')
    expect(surfaceSource).toContain('useThreadMessagesQuery')
    expect(surfaceSource).not.toContain("from '../components/")
    expect(surfaceSource).not.toContain('fetch(')
    expect(surfaceSource).not.toContain('ApiClient')

    expect(resourceOverviewSource).toContain('useSubscriptionsQuery')
    expect(resourceOverviewSource).toContain('useTopSendersQuery')
    expect(resourceOverviewSource).toContain('useCommunicationBlockersQuery')
    expect(resourceOverviewSource).toContain('handleLoadMoreSubscriptions')
    expect(resourceOverviewSource).toContain('handleLoadMoreTopSenders')
  })
})
