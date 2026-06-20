import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramDownloadQueueStatus boundary', () => {
  it('renders current-chat download progress and retry state from derived attachment readiness only', () => {
    const source = readFileSync(new URL('./TelegramDownloadQueueStatus.vue', import.meta.url), 'utf8')

    expect(source).toContain('telegramDownloadQueueItems')
    expect(source).toContain('telegramAttachmentReadiness')
    expect(source).toContain("action_label === 'Retry download'")
    expect(source).toContain("emit('downloadMedia', attachment)")
    expect(source).toContain("t('Retry download')")
    expect(source).not.toContain('useQuery')
    expect(source).not.toContain('fetch(')
  })
})
