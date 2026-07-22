import { describe, expect, it } from 'vitest'
import {
  buildTelegramFolderActionExtras,
  buildTelegramFolderReassignExtras,
  buildTelegramMediaDownloadExtras,
} from './telegramConversationInspectorActions'

describe('telegram conversation inspector command extras', () => {
  it('builds a media download payload from projected media', () => {
    expect(buildTelegramMediaDownloadExtras({
      provider_message_id: 'message-1',
      tdlib_file_id: 42,
      provider_attachment_id: null,
      file_name: 'photo.jpg',
      mime_type: 'image/jpeg',
    })).toEqual({
      providerMessageId: 'message-1',
      tdlibFileId: 42,
      providerAttachmentId: undefined,
      filename: 'photo.jpg',
      contentType: 'image/jpeg',
    })
  })

  it('builds folder add/remove and replace payloads without nullable casts', () => {
    expect(buildTelegramFolderActionExtras(7)).toEqual({ providerFolderId: 7 })
    expect(buildTelegramFolderActionExtras(undefined)).toEqual({ providerFolderId: undefined })
    expect(buildTelegramFolderReassignExtras(7)).toEqual({ providerFolderIds: [7] })
    expect(buildTelegramFolderReassignExtras(undefined)).toEqual({ providerFolderIds: [] })
  })
})
