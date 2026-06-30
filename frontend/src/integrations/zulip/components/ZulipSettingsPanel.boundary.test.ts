import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('ZulipSettingsPanel boundary', () => {
  it('uses Zulip query mutations and backend reference commands instead of direct fetch or file bytes', () => {
    const source = readFileSync(new URL('./ZulipSettingsPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('useSetupZulipBotAccountMutation')
    expect(source).toContain('useEnqueueZulipStreamUploadCommandMutation')
    expect(source).toContain('useEnqueueZulipDirectUploadCommandMutation')
    expect(source).toContain('useEnqueueZulipUploadCommandMutation')
    expect(source).toContain('attachment_id')
    expect(source).toContain('blob_id')
    expect(source).toContain('file bytes stay behind the backend boundary')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('FormData')
    expect(source).not.toContain('type="file"')
  })
})
