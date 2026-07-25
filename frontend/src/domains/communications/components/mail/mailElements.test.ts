import { describe, expect, it } from 'vitest'
import {
  mailListItemAiIndicator,
  mailListItemStatusChipClass,
  type MailListItemModel,
} from './mailElements'

function mailItem(): MailListItemModel {
  return {
    id: 'msg:1',
    accountLabel: 'account-1',
    mailboxLabel: 'Inbox',
    fromName: 'Sender',
    subject: 'Subject',
    snippet: 'Preview',
    timestampLabel: 'now',
    workflowState: 'new',
    aiCategory: 'priority',
  }
}

describe('mailListItemAiIndicator', () => {
  it('presents an available AI-derived category without owning its processing state', () => {
    const indicator = mailListItemAiIndicator(mailItem())

    expect(indicator).toMatchObject({
      label: 'AI',
      tone: 'info',
    })
    expect(indicator?.detail).toContain('summary, category, and evidence')
  })
})

describe('mailListItemStatusChipClass', () => {
  it('only shows the workflow chip for new items', () => {
    expect(mailListItemStatusChipClass(mailItem())).toContain('--visible')
    expect(mailListItemStatusChipClass({ ...mailItem(), workflowState: 'done' }))
      .toBe('mail-list-item__status-chip')
  })
})
