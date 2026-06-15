export type CertificateType =
  | 'smime'
  | 'pgp'
  | 'pdf_sign'
  | 'cades'
  | 'xades'
  | 'gost_sign'
  | 'unknown'

export type CertificateProvider =
  | 'fnmt'
  | 'dnie'
  | 'cryptopro'
  | 'gost'
  | 'apple_keychain'
  | 'pkcs12'
  | 'yubikey'
  | 'usb_token'
  | 'other'

export type CertificateStorageKind =
  | 'os_keychain'
  | 'encrypted_vault'
  | 'pkcs12_file'
  | 'pfx_file'
  | 'smart_card'
  | 'usb_token'
  | 'external_vault'

export type CertificateTrustStatus =
  | 'trusted'
  | 'untrusted'
  | 'expired'
  | 'revoked'
  | 'pending_verification'
  | 'self_signed'

export type MailCertificate = {
  cert_id: string
  owner_name: string
  issuer: string
  serial_number: string | null
  fingerprint_sha256: string | null
  valid_from: string | null
  valid_until: string | null
  cert_type: CertificateType
  provider: CertificateProvider
  storage_kind: CertificateStorageKind
  storage_ref: string | null
  trust_status: CertificateTrustStatus
  is_revoked: boolean
  usage: string[]
  linked_message_id: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type MailCertificateListResponse = {
  items: MailCertificate[]
}

export type MailCertificateCreateRequest = {
  cert_id: string
  owner_name: string
  issuer: string
  serial_number?: string | null
  fingerprint_sha256?: string | null
  valid_from?: string | null
  valid_until?: string | null
  cert_type?: CertificateType
  provider?: CertificateProvider
  storage_kind?: CertificateStorageKind
  storage_ref?: string | null
  trust_status?: CertificateTrustStatus
  is_revoked?: boolean
  usage?: string[]
  linked_message_id?: string | null
  metadata?: Record<string, unknown>
}

export const certificateTypeOptions: CertificateType[] = [
  'smime',
  'pgp',
  'pdf_sign',
  'cades',
  'xades',
  'gost_sign',
  'unknown'
]

export const certificateProviderOptions: CertificateProvider[] = [
  'fnmt',
  'dnie',
  'cryptopro',
  'gost',
  'apple_keychain',
  'pkcs12',
  'yubikey',
  'usb_token',
  'other'
]

export const certificateStorageKindOptions: CertificateStorageKind[] = [
  'os_keychain',
  'encrypted_vault',
  'pkcs12_file',
  'pfx_file',
  'smart_card',
  'usb_token',
  'external_vault'
]

export const certificateTrustStatusOptions: CertificateTrustStatus[] = [
  'trusted',
  'untrusted',
  'expired',
  'revoked',
  'pending_verification',
  'self_signed'
]
