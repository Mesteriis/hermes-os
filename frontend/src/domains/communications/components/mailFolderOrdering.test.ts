import { describe, expect, it } from 'vitest'
import type { CommunicationFolder } from '../types/folders'
import {
  createCommunicationFolderReorderPayload,
  buildCommunicationFolderReorderUpdates,
  mailFolderReorderStatus,
  parseCommunicationFolderReorderPayload
} from './mailFolderOrdering'

function folder(folderId: string, sortOrder: number): CommunicationFolder {
  return {
    folder_id: folderId,
    account_id: null,
    name: folderId,
    description: null,
    color: null,
    sort_order: sortOrder,
    message_count: 0,
    created_at: '2026-06-15T00:00:00Z',
    updated_at: '2026-06-15T00:00:00Z'
  }
}

describe('mail folder ordering', () => {
  it('moves a folder before a target with a single midpoint sort update when there is room', () => {
    const updates = buildCommunicationFolderReorderUpdates([
      folder('alpha', 1000),
      folder('bravo', 2000),
      folder('charlie', 3000)
    ], 'alpha', 'charlie')

    expect(updates).toEqual([{ folderId: 'alpha', sortOrder: 2500 }])
  })

  it('normalizes affected sort orders when no integer gap exists', () => {
    const updates = buildCommunicationFolderReorderUpdates([
      folder('alpha', 0),
      folder('bravo', 1),
      folder('charlie', 2)
    ], 'charlie', 'bravo')

    expect(updates).toEqual([
      { folderId: 'alpha', sortOrder: 1000 },
      { folderId: 'charlie', sortOrder: 2000 },
      { folderId: 'bravo', sortOrder: 3000 }
    ])
  })

  it('does not emit updates for missing folders or no-op moves', () => {
    const folders = [folder('alpha', 1000), folder('bravo', 2000)]

    expect(buildCommunicationFolderReorderUpdates(folders, 'alpha', 'alpha')).toEqual([])
    expect(buildCommunicationFolderReorderUpdates(folders, 'missing', 'alpha')).toEqual([])
    expect(buildCommunicationFolderReorderUpdates(folders, 'alpha', 'missing')).toEqual([])
  })

  it('round-trips typed drag payloads and rejects malformed payloads', () => {
    const payload = createCommunicationFolderReorderPayload(' folder-a ')

    expect(parseCommunicationFolderReorderPayload(payload)).toEqual({
      kind: 'mail-folder-reorder',
      folder_id: 'folder-a'
    })
    expect(parseCommunicationFolderReorderPayload('')).toBeNull()
    expect(parseCommunicationFolderReorderPayload('{"kind":"other","folder_id":"folder-a"}')).toBeNull()
    expect(parseCommunicationFolderReorderPayload('{"kind":"mail-folder-reorder","folder_id":" "}')).toBeNull()
  })

  it('builds reorder feedback from the payload source folder id', () => {
    expect(mailFolderReorderStatus([
      folder('alpha', 1000),
      { ...folder('charlie', 1500), name: 'Charlie' },
      { ...folder('bravo', 2000), name: 'Bravo' }
    ], 'charlie', 'bravo')).toBe('Moved Charlie before Bravo')
  })
})
