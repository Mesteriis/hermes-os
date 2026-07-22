import { describe, expect, it } from 'vitest'
import {
  messengerComposerCapabilityCanOpenFile,
  messengerComposerPlainText,
} from './messengerComposer'

describe('messengerComposerPlainText', () => {
  it('turns a rich editor caption into provider-safe plain text', () => {
    expect(messengerComposerPlainText('<p>Hello <strong>Telegram</strong></p><p>&amp; goodbye</p>'))
      .toBe('Hello Telegram\n& goodbye')
  })

  it('allows the Telegram file capability only while no action is running', () => {
    expect(messengerComposerCapabilityCanOpenFile({ id: 'telegram-file' }, false)).toBe(true)
    expect(messengerComposerCapabilityCanOpenFile({ id: 'telegram-file' }, true)).toBe(false)
    expect(messengerComposerCapabilityCanOpenFile({ id: 'whatsapp-media' }, false)).toBe(false)
  })
})
