import { describe, expect, it } from 'vitest'
import { communicationConversationActiveMessage } from './communicationConversationPresentation'

describe('communication conversation presentation', () => {
  it('selects the latest message and handles an empty conversation', () => {
    expect(communicationConversationActiveMessage([
      { id: 'message-1' },
      { id: 'message-2' },
    ])).toEqual({ id: 'message-2' })
    expect(communicationConversationActiveMessage([])).toBeUndefined()
  })
})
