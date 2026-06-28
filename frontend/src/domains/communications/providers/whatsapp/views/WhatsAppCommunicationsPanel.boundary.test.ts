import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

function readSource(relativePath: string): string {
  return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}

describe('WhatsAppCommunicationsPanel boundary', () => {
  it('uses an in-panel forward target selector', () => {
    const source = readSource('./WhatsAppCommunicationsPanel.vue')
    const detailPaneSource = readSource('./WhatsAppCommunicationsDetailPane.vue')

    expect(source).not.toContain("t('Forward to conversation id')")
    expect(detailPaneSource).toContain("Forward target")
    expect(detailPaneSource).toContain("Filter target conversations")
    expect(detailPaneSource).toContain("Forward here")
  })

  it('uses an in-panel edit flow instead of a prompt', () => {
    const source = readSource('./WhatsAppCommunicationsPanel.vue')
    const detailPaneSource = readSource('./WhatsAppCommunicationsDetailPane.vue')

    expect(source).not.toContain("window.prompt(t('Edit message')")
    expect(detailPaneSource).toContain("Edit draft")
    expect(detailPaneSource).toContain("Edited text")
    expect(detailPaneSource).toContain("Save edit")
  })

  it('supports jumping from pinned and media sections back to timeline messages', () => {
    const source = readSource('./WhatsAppCommunicationsPanel.vue')
    const detailPaneSource = readSource('./WhatsAppCommunicationsDetailPane.vue')

    expect(source).toContain('jumpToMessage')
    expect(detailPaneSource).toContain("Jump to message")
    expect(detailPaneSource).toContain("Open source message")
  })

  it('exposes a dedicated media browsing mode with kind filtering', () => {
    const source = readFileSync(new URL('./WhatsAppCommunicationsPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain("browserMode")
    expect(source).toContain("Timeline")
    expect(source).toContain("All media")
    expect(source).toContain("Images")
    expect(source).toContain("Videos")
    expect(source).toContain("Documents")
  })

  it('renders a safe in-panel media preview surface for projected attachments', () => {
    const source = readSource('./WhatsAppCommunicationsPanel.vue')
    const detailPaneSource = readSource('./WhatsAppCommunicationsDetailPane.vue')

    expect(source).toContain('useAttachmentPreviewQuery')
    expect(detailPaneSource).toContain("Media preview")
    expect(detailPaneSource).toContain("Preview media")
    expect(detailPaneSource).toContain("Select previewable media to open it here.")
    expect(detailPaneSource).toContain("mediaPreview.preview_kind === 'audio'")
    expect(detailPaneSource).toContain("mediaPreview.preview_kind === 'video'")
    expect(detailPaneSource).toContain("mediaPreview.preview_kind === 'pdf'")
  })

  it('renders projected rich WhatsApp message metadata', () => {
    const chatPaneSource = readSource('./WhatsAppCommunicationsChatPane.vue')
    const helpersSource = readSource('./WhatsAppCommunicationsPanel.helpers.ts')

    expect(helpersSource).toContain('whatsapp_link_preview')
    expect(helpersSource).toContain('whatsapp_poll')
    expect(helpersSource).toContain('whatsapp_location')
    expect(helpersSource).toContain('whatsapp_contact_card')
    expect(helpersSource).toContain('whatsapp_sticker')
    expect(helpersSource).toContain('whatsapp_view_once')
    expect(helpersSource).toContain('whatsapp_ephemeral')
    expect(chatPaneSource).toContain('messageLinkPreview')
    expect(chatPaneSource).toContain('messagePollSummary')
  })

  it('renders projected status lifecycle and status-media details in the timeline', () => {
    const chatPaneSource = readSource('./WhatsAppCommunicationsChatPane.vue')
    const helpersSource = readSource('./WhatsAppCommunicationsPanel.helpers.ts')

    expect(helpersSource).toContain('status_view_count')
    expect(helpersSource).toContain('status_last_viewer_display_name')
    expect(helpersSource).toContain('status_deleted_at')
    expect(helpersSource).toContain('status_author_business_profile')
    expect(chatPaneSource).toContain('Status author')
    expect(chatPaneSource).toContain('Status media')
  })
})
