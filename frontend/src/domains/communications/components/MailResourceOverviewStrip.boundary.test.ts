import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MailResourceOverviewStrip boundary', () => {
  it('renders mailbox-level resources without direct API access', () => {
    const source = readFileSync(new URL('./MailResourceOverviewStrip.vue', import.meta.url), 'utf8')

    expect(source).toContain('subscriptions')
    expect(source).toContain('topSenders')
    expect(source).toContain('blockers')
    expect(source).toContain('Newsletters')
    expect(source).toContain('Top senders')
    expect(source).toContain('Blockers')
    expect(source).toContain("from '@tanstack/vue-virtual'")
    expect(source).toContain('useVirtualizer')
    expect(source).toContain('hasMoreSubscriptions')
    expect(source).toContain('hasMoreTopSenders')
    expect(source).toContain('loadMoreSubscriptions')
    expect(source).toContain('loadMoreTopSenders')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
  })
})
