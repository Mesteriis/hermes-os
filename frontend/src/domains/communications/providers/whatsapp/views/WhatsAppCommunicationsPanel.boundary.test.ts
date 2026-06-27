import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('WhatsAppCommunicationsPanel boundary', () => {
  it('uses an in-panel forward target selector', () => {
    const source = readFileSync(new URL('./WhatsAppCommunicationsPanel.vue', import.meta.url), 'utf8')

    expect(source).not.toContain("t('Forward to conversation id')")
    expect(source).toContain("Forward target")
    expect(source).toContain("Filter target conversations")
    expect(source).toContain("Forward here")
  })

  it('uses an in-panel edit flow instead of a prompt', () => {
    const source = readFileSync(new URL('./WhatsAppCommunicationsPanel.vue', import.meta.url), 'utf8')

    expect(source).not.toContain("window.prompt(t('Edit message')")
    expect(source).toContain("Edit draft")
    expect(source).toContain("Edited text")
    expect(source).toContain("Save edit")
  })

  it('supports jumping from pinned and media sections back to timeline messages', () => {
    const source = readFileSync(new URL('./WhatsAppCommunicationsPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('jumpToMessage')
    expect(source).toContain("Jump to message")
    expect(source).toContain("Open source message")
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
    const source = readFileSync(new URL('./WhatsAppCommunicationsPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('useAttachmentPreviewQuery')
    expect(source).toContain("Media preview")
    expect(source).toContain("Preview media")
    expect(source).toContain("Select previewable media to open it here.")
    expect(source).toContain("mediaPreview.preview_kind === 'audio'")
    expect(source).toContain("mediaPreview.preview_kind === 'video'")
    expect(source).toContain("mediaPreview.preview_kind === 'pdf'")
  })

  it('renders projected rich WhatsApp message metadata', () => {
    const source = readFileSync(new URL('./WhatsAppCommunicationsPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('whatsapp_link_preview')
    expect(source).toContain('whatsapp_poll')
    expect(source).toContain('whatsapp_location')
    expect(source).toContain('whatsapp_contact_card')
    expect(source).toContain('whatsapp_sticker')
    expect(source).toContain('whatsapp_view_once')
    expect(source).toContain('whatsapp_ephemeral')
  })

  it('renders projected status lifecycle and status-media details in the timeline', () => {
    const source = readFileSync(new URL('./WhatsAppCommunicationsPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('status_view_count')
    expect(source).toContain('status_last_viewer_display_name')
    expect(source).toContain('status_deleted_at')
    expect(source).toContain('status_author_business_profile')
    expect(source).toContain('Status author')
    expect(source).toContain('Status media')
  })
})
