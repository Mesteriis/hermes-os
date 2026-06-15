import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MailCertificateStrip boundary', () => {
  it('renders certificate inventory and metadata-only creation without direct API access', () => {
    const source = readFileSync(new URL('./MailCertificateStrip.vue', import.meta.url), 'utf8')

    expect(source).toContain('useMailCertificatesQuery')
    expect(source).toContain('useExpiringMailCertificatesQuery')
    expect(source).toContain('useCreateMailCertificateMutation')
    expect(source).toContain('certificateVeeValidationSchema')
    expect(source).toContain('Expiring certificates')
    expect(source).toContain('Add certificate')
    expect(source).toContain('Storage reference')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
  })

})
