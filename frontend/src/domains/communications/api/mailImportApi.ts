import { ApiClient } from '../../../platform/api/ApiClient'
import type { MailImportKind } from '../forms/mailImport'

type EmlImportResponse = {
  message_id: string
  raw_record_id: string
  attachment_count: number
}

type MboxImportResponse = {
  imported_count: number
  message_ids: string[]
  failed_count: number
  failures: Array<{ message_index: number; reason: string }>
}

export type MailImportResult = {
  imported_count: number
  message_ids: string[]
  failed_count: number
  failures: Array<{ message_index: number; reason: string }>
}

export async function importMailFile(
  accountId: string,
  kind: MailImportKind,
  contentBase64: string
): Promise<MailImportResult> {
  if (kind === 'eml') {
    const response = await ApiClient.instance.post<EmlImportResponse>(
      '/api/v1/communications/import/eml',
      { account_id: accountId, eml_base64: contentBase64 },
      'EML import failed'
    )
    return { imported_count: 1, message_ids: [response.message_id], failed_count: 0, failures: [] }
  }

  const response = await ApiClient.instance.post<MboxImportResponse>(
    '/api/v1/communications/import/mbox',
    { account_id: accountId, mbox_base64: contentBase64 },
    'MBOX import failed'
  )
  return {
    imported_count: response.imported_count,
    message_ids: response.message_ids,
    failed_count: response.failed_count,
    failures: response.failures
  }
}
