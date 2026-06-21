import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramAttachmentSearchPanel boundary', () => {
  it('reuses the shared Communication attachment search UI without component-local fetching', () => {
    const source = readFileSync(new URL('./TelegramAttachmentSearchPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain("import AttachmentSearchPanel from '../../../../shared/communications/components/AttachmentSearchPanel.vue'")
    expect(source).toContain('props.accountId?.trim() || null')
    expect(source).toContain('<AttachmentSearchPanel :accountId="normalizedAccountId" />')
    expect(source).toContain("t('Downloaded attachment search')")
    expect(source).toContain('shared Communication attachment index')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient.instance')
  })
})
