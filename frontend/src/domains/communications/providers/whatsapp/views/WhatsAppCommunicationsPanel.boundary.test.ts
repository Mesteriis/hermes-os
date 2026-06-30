import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('WhatsAppCommunicationsPanel boundary', () => {
  it('preserves provider-specific WhatsApp orchestration after removing the legacy Vue render layer', () => {
    const surfaceSource = readFileSync(
      new URL('../../../queries/useWhatsappCommunicationsPanelSurface.ts', import.meta.url),
      'utf8'
    )
    const presentationSource = readFileSync(
      new URL('../../../queries/useWhatsappCommunicationsPresentation.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./WhatsAppCommunicationsPanel.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./WhatsAppCommunicationsChatPane.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./WhatsAppCommunicationsDetailPane.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./WhatsAppCommunicationsPanel.css', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./WhatsAppCommunicationsPanel.helpers.ts', import.meta.url))).toBe(false)

    expect(surfaceSource).toContain('useWhatsappBusinessConversationsQuery')
    expect(surfaceSource).toContain('useWhatsappBusinessMessagesQuery')
    expect(surfaceSource).toContain('useWhatsappMediaSearchQuery')
    expect(surfaceSource).toContain('useAttachmentPreviewQuery')
    expect(surfaceSource).toContain('beginForwardMessage')
    expect(surfaceSource).toContain('confirmForwardMessage')
    expect(surfaceSource).toContain('confirmEditMessage')
    expect(surfaceSource).toContain('toggleConversationArchive')
    expect(surfaceSource).toContain('toggleConversationMute')
    expect(surfaceSource).toContain('toggleConversationPin')
    expect(surfaceSource).toContain('toggleConversationUnread')
    expect(surfaceSource).toContain('addReaction')
    expect(surfaceSource).toContain('removeReaction')
    expect(surfaceSource).toContain('jumpToMessage')
    expect(surfaceSource).not.toContain("../providers/whatsapp/views/")
    expect(surfaceSource).not.toContain('.vue')

    expect(presentationSource).toContain('whatsapp_link_preview')
    expect(presentationSource).toContain('whatsapp_poll')
    expect(presentationSource).toContain('whatsapp_location')
    expect(presentationSource).toContain('whatsapp_contact_card')
    expect(presentationSource).toContain('whatsapp_sticker')
    expect(presentationSource).toContain('whatsapp_view_once')
    expect(presentationSource).toContain('whatsapp_ephemeral')
    expect(presentationSource).toContain('status_view_count')
    expect(presentationSource).toContain('status_last_viewer_display_name')
    expect(presentationSource).toContain('status_deleted_at')
    expect(presentationSource).toContain('status_author_business_profile')
    expect(presentationSource).toContain('messageLinkPreview')
    expect(presentationSource).toContain('messagePollSummary')
    expect(presentationSource).toContain('statusMediaCountLabel')
    expect(presentationSource).toContain('statusAuthorHeadline')
    expect(presentationSource).not.toContain('.vue')
  })
})
