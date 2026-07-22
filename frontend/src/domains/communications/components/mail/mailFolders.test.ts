import { describe, expect, it } from 'vitest'
import {
  mailFolderDepthClass,
  mailFolderIsActive,
  mailFolderLocalizedAriaLabel,
  mailFolderToggleAriaLabel,
  type MailFolderModel,
} from './mailFolders'

describe('mail folder presentation', () => {
  it('projects active, depth and localized accessibility state', () => {
    const folder: MailFolderModel = { id: 'inbox', kind: 'inbox', label: 'Inbox', count: 4, unreadCount: 2 }
    const row = { folder, depth: 6, hasChildren: true, expanded: false }
    const t = (key: string, params?: Record<string, string | number>) =>
      params ? `${key}:${Object.values(params).join(',')}` : key

    expect(mailFolderIsActive(folder, 'inbox')).toBe(true)
    expect(mailFolderDepthClass(row)).toBe('mail-folder-list__item--depth-4')
    expect(mailFolderLocalizedAriaLabel(folder, t)).toBe('Inbox, {count} unread:2, {count} total:4')
    expect(mailFolderToggleAriaLabel(row, t)).toBe('Expand folder: Inbox')
  })
})
