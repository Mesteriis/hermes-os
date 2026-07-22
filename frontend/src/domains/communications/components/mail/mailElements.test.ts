import { describe, expect, it } from 'vitest'
import {
  mailListItemAiIndicator,
  mailListItemStatusChipClass,
  type MailListItemModel,
} from './mailElements'

function failedAiItem(): MailListItemModel {
  return {
    id: 'msg:1',
    accountLabel: 'account-1',
    mailboxLabel: 'Inbox',
    fromName: 'Sender',
    subject: 'Subject',
    snippet: 'Preview',
    timestampLabel: 'now',
    workflowState: 'new',
    aiState: 'FAILED',
  }
}

describe('mailListItemAiIndicator', () => {
  it('does not present every failed AI state as a retry', () => {
    const indicator = mailListItemAiIndicator(failedAiItem())

    expect(indicator).toMatchObject({
      label: 'AI attention',
      tone: 'warning',
    })
    expect(indicator?.detail).toContain('retry or review state')
  })
})

describe('mailListItemStatusChipClass', () => {
  it('only shows the workflow chip for new items', () => {
    expect(mailListItemStatusChipClass(failedAiItem())).toContain('--visible')
    expect(mailListItemStatusChipClass({ ...failedAiItem(), workflowState: 'done' }))
      .toBe('mail-list-item__status-chip')
  })
})
