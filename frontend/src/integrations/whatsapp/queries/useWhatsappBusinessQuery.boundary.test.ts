import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('WhatsApp business-query boundary', () => {
  it('keeps provider operational queries inside the integration unit', () => {
    const source = readFileSync(new URL('./useWhatsappBusinessQuery.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('../../../../domains/communications/queries/whatsappBusinessQueries.ts', import.meta.url))).toBe(false)
    expect(source).toContain('useWhatsappBusinessConversationsQuery')
    expect(source).toContain('useSendWhatsappMessageMutation')
    expect(source).toContain('useMarkWhatsappConversationUnreadMutation')
    expect(source).not.toContain('domains/communications')
  })
})
