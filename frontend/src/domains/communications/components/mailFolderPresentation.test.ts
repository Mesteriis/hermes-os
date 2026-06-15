import { describe, expect, it } from 'vitest'
import {
  createChildFolderDraft,
  deriveMailFolderDisplayRow,
  mailFolderHierarchyDeleteImpact,
  orderMailFolderDisplayRows
} from './mailFolderPresentation'
import type { MailFolder } from '../types/folders'

function folder(overrides: Partial<MailFolder>): MailFolder {
  return {
    folder_id: 'folder-1',
    account_id: 'account-1',
    name: 'Inbox',
    description: null,
    color: '#3b82f6',
    sort_order: 0,
    message_count: 4,
    created_at: '2026-06-15T00:00:00Z',
    updated_at: '2026-06-15T00:00:00Z',
    ...overrides
  }
}

describe('mail folder presentation helpers', () => {
  it('parses root folder names as a single depth entry', () => {
    const row = deriveMailFolderDisplayRow(folder({ name: 'Inbox' }))

    expect(row.depth).toBe(0)
    expect(row.leafName).toBe('Inbox')
    expect(row.pathPrefix).toBe('')
  })

  it('derives path depth, leaf and prefix from slash-delimited names', () => {
    const row = deriveMailFolderDisplayRow(folder({
      folder_id: 'folder-2',
      name: 'Projects / Client A / Q1'
    }))

    expect(row.depth).toBe(2)
    expect(row.leafName).toBe('Q1')
    expect(row.pathPrefix).toBe('Projects / Client A')
  })

  it('normalizes blank segments and trims whitespace in folder paths', () => {
    const row = deriveMailFolderDisplayRow(folder({
      folder_id: 'folder-3',
      name: '  Archives //  2026 // ' 
    }))

    expect(row.depth).toBe(1)
    expect(row.leafName).toBe('2026')
    expect(row.pathPrefix).toBe('Archives')
  })

  it('orders folders by sort order and hierarchy so parents stay ahead of children', () => {
    const rows = orderMailFolderDisplayRows([
      folder({ folder_id: 'folder-4', name: 'Projects / Client A / Q1', sort_order: 100 }),
      folder({ folder_id: 'folder-2', name: 'Projects', sort_order: 100 }),
      folder({ folder_id: 'folder-3', name: 'Projects / Client A', sort_order: 100 }),
      folder({ folder_id: 'folder-1', name: 'Archive', sort_order: 50 })
    ])

    expect(rows.map((row) => row.folder.folder_id)).toEqual([
      'folder-1',
      'folder-2',
      'folder-3',
      'folder-4'
    ])
  })

  it('builds a child-folder draft from the selected parent path', () => {
    expect(createChildFolderDraft(folder({
      folder_id: 'folder-5',
      name: 'Projects / Client A',
      sort_order: 320
    }))).toEqual({
      parentPath: 'Projects / Client A',
      sortOrder: 320
    })
  })

  it('reports descendant impact for hierarchy-aware delete warnings', () => {
    expect(mailFolderHierarchyDeleteImpact([
      folder({ folder_id: 'root', name: 'Projects' }),
      folder({ folder_id: 'child-1', name: 'Projects / Client A' }),
      folder({ folder_id: 'child-2', name: 'Projects / Client B' }),
      folder({ folder_id: 'grandchild', name: 'Projects / Client A / Q1' }),
      folder({ folder_id: 'other', name: 'Archive' })
    ], 'root')).toEqual({
      descendantCount: 3,
      descendantLeafNames: ['Client A', 'Q1', 'Client B']
    })
  })
})
