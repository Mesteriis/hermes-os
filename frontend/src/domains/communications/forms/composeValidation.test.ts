import { describe, expect, it } from 'vitest'
import {
  composeSendSchema,
  splitComposeRecipients,
  toComposeValidationValues
} from './composeValidation'

describe('compose validation', () => {
  it('splits comma-separated recipients and extracts angle-bracket addresses', () => {
    expect(splitComposeRecipients('Alex <alex@example.com>, team@example.org')).toEqual([
      'alex@example.com',
      'team@example.org'
    ])
  })

  it('accepts a valid send form', () => {
    const parsed = composeSendSchema.parse({
      accountId: 'account-1',
      toText: 'recipient@example.com',
      ccText: '',
      bccText: '',
      subject: 'Quarterly update',
      body: 'Hello',
      inReplyTo: null
    })

    expect(parsed.toText).toBe('recipient@example.com')
  })

  it('rejects missing account and invalid recipients', () => {
    const result = composeSendSchema.safeParse({
      accountId: '',
      toText: 'not-an-email',
      ccText: 'copy@example.com',
      bccText: '',
      subject: 'Quarterly update',
      body: 'Hello',
      inReplyTo: null
    })

    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.error.issues.map((issue) => issue.path.join('.'))).toEqual([
        'accountId',
        'toText'
      ])
    }
  })

  it('maps the store compose model into validation values', () => {
    expect(toComposeValidationValues({
      mode: 'reply',
      draftId: 'draft-1',
      accountId: 'account-1',
      toText: 'recipient@example.com',
      ccText: '',
      bccText: '',
      subject: 'Re: Update',
      body: 'Thanks',
      bodyHtml: null,
      bodyFormat: 'plain',
      scheduledSendAt: '',
      undoSendSeconds: null,
      inReplyTo: 'provider-message-1'
    })).toEqual({
      accountId: 'account-1',
      toText: 'recipient@example.com',
      ccText: '',
      bccText: '',
      subject: 'Re: Update',
      body: 'Thanks',
      inReplyTo: 'provider-message-1'
    })
  })
})
