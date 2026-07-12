import { bytesToBase64 } from './composeAttachmentUpload'

export const MAX_MAIL_IMPORT_BYTES = 20 * 1024 * 1024

export type MailImportKind = 'eml' | 'mbox'

type MailImportFileMetadata = {
  name: string
  size: number
}

export function mailImportKindForFile(file: MailImportFileMetadata): MailImportKind {
  if (file.size <= 0) throw new Error('Mail import file is empty')
  if (file.size > MAX_MAIL_IMPORT_BYTES) {
    throw new Error('Mail import exceeds the 20 MiB limit')
  }

  const name = file.name.trim().toLowerCase()
  if (name.endsWith('.eml')) return 'eml'
  if (name.endsWith('.mbox')) return 'mbox'
  throw new Error('Choose an .eml or .mbox file')
}

export async function prepareMailImport(file: File): Promise<{ kind: MailImportKind; contentBase64: string }> {
  const kind = mailImportKindForFile(file)
  return {
    kind,
    contentBase64: bytesToBase64(new Uint8Array(await file.arrayBuffer()))
  }
}
