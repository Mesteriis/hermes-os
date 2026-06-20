import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramFolderMembershipBar folder actions', () => {
  it('exposes explicit add/remove folder emits without component-level fetches', () => {
    const source = readFileSync(new URL('./TelegramFolderMembershipBar.vue', import.meta.url), 'utf8')

    expect(source).toContain('addFolder: [providerFolderId: number]')
    expect(source).toContain('removeFolder: [providerFolderId: number]')
    expect(source).toContain("t('No Telegram folders')")
    expect(source).toContain("emit('addFolder'")
    expect(source).toContain("emit('removeFolder'")
    expect(source).not.toContain('fetch(')
  })
})
