import { describe, expect, it } from 'vitest'
import { mailActionGroupDefaultAction, type MailActionMenuGroup } from './mailActions'

describe('mail action group default action', () => {
  const group = (tone?: MailActionMenuGroup['tone']): MailActionMenuGroup => ({
    id: 'state',
    label: 'State',
    menuLabel: 'Open state actions',
    icon: 'tabler:activity',
    tone,
    items: [
      { id: 'disabled-action', label: 'Disabled', disabled: true },
      { id: 'available-action', label: 'Available' },
    ],
  })

  it('selects the first enabled action for a regular group', () => {
    expect(mailActionGroupDefaultAction(group('info'))).toBe('available-action')
  })

  it('does not default-trigger destructive groups', () => {
    expect(mailActionGroupDefaultAction(group('danger'))).toBeUndefined()
  })
})
