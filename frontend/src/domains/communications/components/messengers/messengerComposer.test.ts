import { describe, expect, it } from 'vitest'
import { messengerComposerPlainText } from './messengerComposer'

describe('messengerComposerPlainText', () => {
  it('turns a rich editor caption into provider-safe plain text', () => {
    expect(messengerComposerPlainText('<p>Hello <strong>Telegram</strong></p><p>&amp; goodbye</p>'))
      .toBe('Hello Telegram\n& goodbye')
  })
})
