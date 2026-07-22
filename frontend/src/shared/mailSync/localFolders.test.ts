import { afterEach, describe, expect, it, vi } from 'vitest'
import { getCommunicationsConnectClient } from '../../platform/connect/communicationsClient'
import { MAX_MAIL_BATCH_SIZE } from './types'
import { fetchMailLocalFolders } from './localFolders'

vi.mock('../../platform/connect/communicationsClient', () => ({
  getCommunicationsConnectClient: vi.fn(),
}))

describe('mail local folders', () => {
  afterEach(() => vi.resetAllMocks())

  it('lists only local folders belonging to the selected mail account', async () => {
    const listFolders = vi.fn().mockResolvedValue({
      items: [
        {
          folderId: 'mail-folder:work',
          accountId: 'mail-account:1',
          name: 'Work',
          sortOrder: 5,
          messageCount: 10n,
          createdAt: '2026-07-11T10:00:00Z',
          updatedAt: '2026-07-11T10:00:00Z',
        },
      ],
    })
    vi.mocked(getCommunicationsConnectClient).mockReturnValue({ listFolders } as never)

    await expect(fetchMailLocalFolders('mail-account:1')).resolves.toEqual([
      { folder_id: 'mail-folder:work', name: 'Work' },
    ])
    expect(listFolders).toHaveBeenCalledWith({
      accountId: 'mail-account:1',
      page: { limit: MAX_MAIL_BATCH_SIZE, cursor: '' },
    })
  })
})
