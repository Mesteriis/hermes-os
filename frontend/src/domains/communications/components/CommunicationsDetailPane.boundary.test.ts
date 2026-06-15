import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationsDetailPane boundary', () => {
  it('forwards bilingual reply send events from the mail viewer', () => {
    const source = readFileSync(
      new URL('./CommunicationsDetailPane.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('sendBilingualReply')
    expect(source).toContain('exportMessage')
    expect(source).toContain('addLabel')
    expect(source).toContain('removeLabel')
    expect(source).toContain('snoozeMessage')
    expect(source).toContain('replyAll')
    expect(source).toContain('forwardMessage')
    expect(source).toContain('redirectMessage')
    expect(source).toContain('markMessageRead')
    expect(source).toContain('markMessageUnread')
    expect(source).toContain('deleteFromProvider')
    expect(source).toContain('MailViewer')
    expect(source).toContain('@send-bilingual-reply')
    expect(source).toContain('@export-message')
    expect(source).toContain('@add-label')
    expect(source).toContain('@remove-label')
    expect(source).toContain('@snooze-message')
    expect(source).toContain('@mark-message-read')
    expect(source).toContain('@mark-message-unread')
    expect(source).toContain('@delete-from-provider')
    expect(source).toContain('@reply-all')
    expect(source).toContain('@forward-message')
    expect(source).toContain('@redirect-message')
  })

  it('renders selected threads through the conversation timeline instead of the single-message viewer', () => {
    const source = readFileSync(
      new URL('./CommunicationsDetailPane.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('ThreadConversationView')
    expect(source).toContain('selectedThread')
    expect(source).toContain('threadMessages')
    expect(source).toContain('isThreadReplySending')
    expect(source).toContain(':is-sending-reply="isThreadReplySending"')
    expect(source).toContain('@open-message')
    expect(source).toContain('@reply-to-message')
    expect(source).toContain('@save-reply-draft')
    expect(source).toContain('@send-reply')
  })
})
