import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationsActionBar export boundary', () => {
  it('surfaces the latest message export as a browser download link', () => {
    const source = readFileSync(new URL('./CommunicationsActionBar.vue', import.meta.url), 'utf8')

    expect(source).toContain('lastMessageExport')
    expect(source).toContain('messageExportDownloadHref')
    expect(source).toContain('download')
    expect(source).toContain('Export ready')
    expect(source).toContain('MailResourceOverviewStrip')
    expect(source).toContain('MailSyncSettingsStrip')
    expect(source).toContain('../../../shared/mailSync/MailSyncSettingsStrip.vue')
    expect(source).toContain('MailCertificateStrip')
    expect(source).toContain('hasMoreDrafts')
    expect(source).toContain('isLoadingMoreDrafts')
    expect(source).toContain('loadMoreDrafts')
    expect(source).toContain('syncSettings')
    expect(source).toContain('updateSyncSettings')
    expect(source).toContain('subscriptions')
    expect(source).toContain('topSenders')
    expect(source).toContain('blockers')
    expect(source).toContain('hasMoreSubscriptions')
    expect(source).toContain('isLoadingMoreSubscriptions')
    expect(source).toContain('hasMoreTopSenders')
    expect(source).toContain('isLoadingMoreTopSenders')
    expect(source).toContain('loadMoreSubscriptions')
    expect(source).toContain('loadMoreTopSenders')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
  })
})
