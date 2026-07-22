import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type { AttachmentSearchRequest } from '../types/attachments'

export const attachmentScanStatusOptions = [
  'not_scanned',
  'clean',
  'suspicious',
  'malicious',
  'failed'
] as const

export const attachmentSearchFormSchema = z.object({
  query: z.string().trim().max(500, 'Search query is too long'),
  content_type: z.string().trim().max(120, 'Content type is too long'),
  scan_status: z.union([z.literal(''), z.enum(attachmentScanStatusOptions)])
})

export type AttachmentSearchFormValues = z.infer<typeof attachmentSearchFormSchema>

export const attachmentSearchVeeValidationSchema = toTypedSchema(attachmentSearchFormSchema)

export function attachmentSearchFormDefaults(): AttachmentSearchFormValues {
  return {
    query: '',
    content_type: '',
    scan_status: ''
  }
}

export function attachmentSearchFormToRequest(
  values: AttachmentSearchFormValues,
  accountId: string | null
): AttachmentSearchRequest {
  const parsed = attachmentSearchFormSchema.parse(values)
  const request: AttachmentSearchRequest = { limit: 50 }
  if (accountId?.trim()) request.account_id = accountId.trim()
  if (parsed.query) request.q = parsed.query
  if (parsed.content_type) request.content_type = parsed.content_type
  if (parsed.scan_status) request.scan_status = parsed.scan_status
  return request
}
