import { describe, expect, it } from 'vitest'
import type { FolderMessage } from '../types/folders'
import { folderMessagesToMailSummaries } from './folderMailList'

function folderMessage(overrides: Partial<FolderMessage> = {}): FolderMessage {
  return {
    folder_id: 'folder-1',
    message_id: 'msg-1',
    account_id: 'account-1',
    subject: 'Quarterly update',
    sender: 'alice@example.com',
    occurred_at: null,
    projected_at: '2026-06-15T10:00:00Z',
    workflow_state: 'waiting',
    local_state: 'active',
    added_at: '2026-06-15T10:05:00Z',
    attachment_count: 3,
    ...overrides
  }
}

describe('folder mail list mapping', () => {
  it('maps cursor-paginated folder message rows into existing mail list summaries', () => {
    const [summary] = folderMessagesToMailSummaries([folderMessage()])

    expect(summary).toMatchObject({
      message_id: 'msg-1',
      account_id: 'account-1',
      subject: 'Quarterly update',
      sender: 'alice@example.com',
      occurred_at: null,
      projected_at: '2026-06-15T10:00:00Z',
      channel_kind: 'email',
      workflow_state: 'waiting',
      local_state: 'active',
      attachment_count: 3
    })
    expect(summary.recipients).toEqual([])
    expect(summary.body_text_preview).toBe('')
    expect(summary.message_metadata).toEqual({
      folder_id: 'folder-1',
      folder_added_at: '2026-06-15T10:05:00Z'
    })
  })
})
