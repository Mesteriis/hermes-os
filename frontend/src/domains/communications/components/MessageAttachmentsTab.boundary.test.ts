import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('MessageAttachmentsTab boundary', () => {
  it('preserves attachment preview, translation, and archive inspection contracts after removing the attachments tab render layer', () => {
    const workspaceQuerySource = readFileSync(
      new URL('../queries/mailWorkspaceQueries.ts', import.meta.url),
      'utf8'
    )
    const attachmentSource = readFileSync(new URL('./attachmentTable.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./MessageAttachmentsTab.vue', import.meta.url))).toBe(false)
    expect(workspaceQuerySource).toContain('export function useAttachmentArchiveInspectionQuery')
    expect(workspaceQuerySource).toContain('export function useAttachmentPreviewQuery')
    expect(workspaceQuerySource).toContain('export function useTranslateAttachmentMutation()')
    expect(attachmentSource).toContain('isInspectableArchiveAttachment')
    expect(attachmentSource).toContain('isPreviewableImageAttachment')
    expect(attachmentSource).toContain('isPreviewablePdfAttachment')
    expect(attachmentSource).toContain('isPreviewableAttachment')
  })
})
