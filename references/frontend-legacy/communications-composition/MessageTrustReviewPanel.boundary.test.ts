// Historical pre-clean-room orchestration test. Not part of the active validation suite.
import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('MessageTrustReviewPanel boundary', () => {
  it('preserves security and recipient review handlers after removing the trust review render layer', () => {
    const surfaceSource = readFileSync(
      new URL('../queries/useCommunicationsPageSurface.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./MessageTrustReviewPanel.vue', import.meta.url))).toBe(false)
    expect(surfaceSource).toContain('handleReviewSecurity')
    expect(surfaceSource).toContain('handleReviewRecipients')
  })
})
