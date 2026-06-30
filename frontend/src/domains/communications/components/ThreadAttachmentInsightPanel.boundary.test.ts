import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('ThreadAttachmentInsightPanel boundaries', () => {
  it('preserves attachment inspection query hooks and preview helpers after removing the thread attachment panel', () => {
    const workspaceQuerySource = readFileSync(
      new URL('../queries/mailWorkspaceQueries.ts', import.meta.url),
      'utf8'
    )
    const attachmentSource = readFileSync(new URL('./attachmentTable.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./ThreadAttachmentInsightPanel.vue', import.meta.url))).toBe(false)
    expect(workspaceQuerySource).toContain('export function useAttachmentArchiveInspectionQuery')
    expect(workspaceQuerySource).toContain('export function useAttachmentPreviewQuery')
    expect(workspaceQuerySource).toContain('export function useTranslateAttachmentMutation()')
    expect(attachmentSource).toContain('isPreviewableAttachment')
    expect(attachmentSource).toContain('isInspectableArchiveAttachment')
    expect(attachmentSource).toContain('isPreviewableImageAttachment')
  })
})
