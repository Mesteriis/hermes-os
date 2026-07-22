import { describe, expect, it } from 'vitest'
import { findSelectedSettingsTreeItem } from './settingsPagePresentation'

describe('settings page selectors', () => {
  it('finds a selected item without owning reactive state', () => {
    const item = { id: 'accounts' as const, label: 'Accounts', description: 'Accounts', icon: 'tabler:id' }
    expect(findSelectedSettingsTreeItem([{ label: 'Workspace', items: [item] }], 'accounts')).toBe(item)
    expect(findSelectedSettingsTreeItem([{ label: 'Workspace', items: [item] }], 'ai')).toBeNull()
  })
})
