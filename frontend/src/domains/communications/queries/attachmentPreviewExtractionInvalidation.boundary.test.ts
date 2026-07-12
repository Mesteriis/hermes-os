import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('attachment preview extraction invalidation boundary', () => {
  it('refreshes a safe preview after local extraction creates a derived artifact', () => {
    const source = readFileSync(new URL('./mailWorkspaceQueries.ts', import.meta.url), 'utf8')

    expect(source).toContain("['communications-attachment-extracted-text', attachmentId]")
    expect(source).toContain("['communications-attachment-preview', attachmentId]")
  })
})
