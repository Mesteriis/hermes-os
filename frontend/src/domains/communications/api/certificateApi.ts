import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  MailCertificate,
  MailCertificateCreateRequest,
  MailCertificateListResponse
} from '../types/certificates'

export async function fetchMailCertificates(): Promise<MailCertificateListResponse> {
  return ApiClient.instance.get<MailCertificateListResponse>(
    '/api/v1/communications/certificates',
    'Mail certificates request failed'
  )
}

export async function fetchExpiringMailCertificates(days = 90): Promise<MailCertificateListResponse> {
  const safeDays = Math.min(Math.max(Math.trunc(days), 1), 3650)
  return ApiClient.instance.get<MailCertificateListResponse>(
    `/api/v1/communications/certificates/expiring?days=${safeDays}`,
    'Expiring mail certificates request failed'
  )
}

export async function createMailCertificate(
  request: MailCertificateCreateRequest
): Promise<MailCertificate> {
  return ApiClient.instance.post<MailCertificate>(
    '/api/v1/communications/certificates',
    request,
    'Mail certificate save failed'
  )
}
