import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('mail certificate query boundary', () => {
  it('wraps certificate API through TanStack Query hooks', () => {
    const source = readFileSync(new URL('./mailWorkspaceQueries.ts', import.meta.url), 'utf8')

    expect(source).toContain('useMailCertificatesQuery')
    expect(source).toContain('useExpiringMailCertificatesQuery')
    expect(source).toContain('useCreateMailCertificateMutation')
    expect(source).toContain('fetchMailCertificates')
    expect(source).toContain('fetchExpiringMailCertificates')
    expect(source).toContain('createMailCertificate')
    expect(source).toContain("['communications-certificates']")
    expect(source).toContain("['communications-certificates-expiring'")
  })

})
