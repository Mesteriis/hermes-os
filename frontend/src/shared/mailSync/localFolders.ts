import { getCommunicationsConnectClient } from '../../platform/connect/communicationsClient'
import { MAX_MAIL_BATCH_SIZE } from './types'

export type MailLocalFolder = {
  folder_id: string
  name: string
}

export async function fetchMailLocalFolders(accountId: string): Promise<MailLocalFolder[]> {
  const response = await getCommunicationsConnectClient().listFolders({
    accountId,
    page: { limit: MAX_MAIL_BATCH_SIZE, cursor: '' },
  })

  return response.items
    .filter((item) => item.accountId === accountId)
    .map((item) => ({ folder_id: item.folderId, name: item.name }))
}
