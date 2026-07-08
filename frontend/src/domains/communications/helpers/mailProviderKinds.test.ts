import { describe, expect, it } from 'vitest'

import { isMailProviderKind } from './mailProviderKinds'

describe('mail provider kinds', () => {
  it('accepts only email provider accounts for mail surfaces', () => {
    expect(isMailProviderKind('gmail')).toBe(true)
    expect(isMailProviderKind('icloud')).toBe(true)
    expect(isMailProviderKind('imap')).toBe(true)
    expect(isMailProviderKind(' telegram_user ')).toBe(false)
    expect(isMailProviderKind('whatsapp_web')).toBe(false)
    expect(isMailProviderKind('')).toBe(false)
  })
})
