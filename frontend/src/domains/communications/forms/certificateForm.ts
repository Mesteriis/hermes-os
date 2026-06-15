import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import {
  certificateProviderOptions,
  certificateStorageKindOptions,
  certificateTrustStatusOptions,
  certificateTypeOptions,
  type CertificateProvider,
  type CertificateStorageKind,
  type CertificateTrustStatus,
  type CertificateType,
  type MailCertificateCreateRequest
} from '../types/certificates'

const optionalTrimmed = z.string().trim().optional().transform((value) => value || undefined)

export const certificateFormSchema = z.object({
  cert_id: z.string().trim().min(1, 'Certificate id is required'),
  owner_name: z.string().trim().min(1, 'Owner is required'),
  issuer: z.string().trim().min(1, 'Issuer is required'),
  fingerprint_sha256: optionalTrimmed,
  valid_until: optionalTrimmed,
  cert_type: z.enum(certificateTypeOptions as [CertificateType, ...CertificateType[]]),
  provider: z.enum(certificateProviderOptions as [CertificateProvider, ...CertificateProvider[]]),
  storage_kind: z.enum(certificateStorageKindOptions as [CertificateStorageKind, ...CertificateStorageKind[]]),
  storage_ref: optionalTrimmed,
  trust_status: z.enum(certificateTrustStatusOptions as [CertificateTrustStatus, ...CertificateTrustStatus[]]),
  usage: optionalTrimmed
})

export type CertificateFormValues = z.input<typeof certificateFormSchema>

export const certificateVeeValidationSchema = toTypedSchema(certificateFormSchema)

export function certificateFormDefaults(): CertificateFormValues {
  return {
    cert_id: '',
    owner_name: '',
    issuer: '',
    fingerprint_sha256: '',
    valid_until: '',
    cert_type: 'smime',
    provider: 'other',
    storage_kind: 'encrypted_vault',
    storage_ref: '',
    trust_status: 'pending_verification',
    usage: 'signing, encryption'
  }
}

export function certificateFormToCreateRequest(values: CertificateFormValues): MailCertificateCreateRequest {
  const parsed = certificateFormSchema.parse(values)
  return {
    cert_id: parsed.cert_id,
    owner_name: parsed.owner_name,
    issuer: parsed.issuer,
    fingerprint_sha256: parsed.fingerprint_sha256 ?? null,
    valid_until: localDateTimeToIso(parsed.valid_until),
    cert_type: parsed.cert_type,
    provider: parsed.provider,
    storage_kind: parsed.storage_kind,
    storage_ref: parsed.storage_ref ?? null,
    trust_status: parsed.trust_status,
    usage: splitUsage(parsed.usage),
    metadata: {}
  }
}

function splitUsage(value: string | undefined): string[] {
  if (!value) return []
  return value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean)
}

function localDateTimeToIso(value: string | undefined): string | null {
  if (!value) return null
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return null
  return date.toISOString()
}
