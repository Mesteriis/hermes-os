import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('attachment translation mutation boundary', () => {
  it('routes attachment translation through TanStack mutation and the communications API client', () => {
    const source = readFileSync(new URL('./mailWorkspaceQueries.ts', import.meta.url), 'utf8')

    expect(source).toContain('translateAttachment')
    expect(source).toContain('AttachmentTranslationResponse')
    expect(source).toContain('export function useTranslateAttachmentMutation()')
    expect(source).toContain('useMutation<')
    expect(source).toContain('translateAttachment(attachmentId, request)')
  })
})
