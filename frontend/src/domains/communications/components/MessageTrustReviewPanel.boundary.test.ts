import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MessageTrustReviewPanel boundary', () => {
  it('renders security and recipient review controls without direct API access', () => {
    const source = readFileSync(new URL('./MessageTrustReviewPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('Security Review')
    expect(source).toContain('Recipient Suggestions')
    expect(source).toContain('reviewSecurity')
    expect(source).toContain('reviewRecipients')
    expect(source).toContain('authRisk')
    expect(source).toContain('smartCc')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
  })
})
