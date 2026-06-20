import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationsListPane folder browsing boundary', () => {
  it('can force the virtualized MailList for custom folder message results', () => {
    const source = readFileSync(
      new URL('./CommunicationsListPane.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('isFolderMode: boolean')
    expect(source).toContain('accountId: string')
    expect(source).toContain('threads: CommunicationThreadSummary[]')
    expect(source).toContain('selectedThreadId: string')
    expect(source).toContain('hasThreadNextPage: boolean')
    expect(source).toContain('selectThread: [thread: CommunicationThreadSummary]')
    expect(source).toContain('@load-more-threads="emit(')
    expect(source).toContain('v-else-if="!isFolderMode && (navigatorMode ===')
    expect(source).toContain('<MailList')
    expect(source).toContain(':account-id="accountId"')
    expect(source).toContain('@load-more="emit(')
  })

  it('forwards mail list keyboard multi-select commands', () => {
    const source = readFileSync(
      new URL('./CommunicationsListPane.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('selectVisible: [messageIds: string[]]')
    expect(source).toContain('clearSelection: []')
    expect(source).toContain('@select-visible="emit(\'selectVisible\', $event)"')
    expect(source).toContain('@clear-selection="emit(\'clearSelection\')"')
  })
})
