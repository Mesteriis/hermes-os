import { describe, expect, it } from 'vitest'
import {
  MAIL_MESSAGE_DRAG_TYPE,
  createMailMessageDragPayload,
  hasMailMessageDragType,
  parseMailMessageDragPayload
} from './mailDragDrop'

describe('mail drag/drop helpers', () => {
  it('serializes and parses selected mail message drag payloads', () => {
    const payload = createMailMessageDragPayload(' msg-1 ', [' msg-2 ', 'msg-1', ''])

    expect(parseMailMessageDragPayload(payload)).toEqual({
      kind: 'mail-message-selection',
      message_id: 'msg-1',
      message_ids: ['msg-1', 'msg-2']
    })
  })

  it('keeps compatibility with legacy single-message payloads', () => {
    const payload = JSON.stringify({ kind: 'mail-message-selection', message_id: 'msg-1' })

    expect(parseMailMessageDragPayload(payload)).toEqual({
      kind: 'mail-message-selection',
      message_id: 'msg-1',
      message_ids: ['msg-1']
    })
  })

  it('rejects malformed drag payloads', () => {
    expect(parseMailMessageDragPayload('')).toBeNull()
    expect(parseMailMessageDragPayload('not-json')).toBeNull()
    expect(parseMailMessageDragPayload(JSON.stringify({ kind: 'other', message_id: 'msg-1' }))).toBeNull()
    expect(parseMailMessageDragPayload(JSON.stringify({ kind: 'mail-message-selection', message_id: '' }))).toBeNull()
    expect(parseMailMessageDragPayload(JSON.stringify({ kind: 'mail-message-selection', message_id: 'msg-1', message_ids: [''] }))).toBeNull()
  })

  it('detects the custom Hermes mail drag type', () => {
    expect(hasMailMessageDragType([MAIL_MESSAGE_DRAG_TYPE, 'text/plain'])).toBe(true)
    expect(hasMailMessageDragType(['text/plain'])).toBe(false)
  })
})
