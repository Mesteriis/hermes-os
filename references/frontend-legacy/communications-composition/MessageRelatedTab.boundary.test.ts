// Historical pre-clean-room orchestration test. Not part of the active validation suite.
import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('MessageRelatedTab export boundary', () => {
  it('preserves related-message metadata helpers and action handlers after removing the related tab render layer', () => {
    const pageModelSource = readFileSync(
      new URL('../helpers/communicationPageModels.ts', import.meta.url),
      'utf8'
    )
    const surfaceSource = readFileSync(
      new URL('../queries/useCommunicationsPageSurface.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./MessageRelatedTab.vue', import.meta.url))).toBe(false)
    expect(pageModelSource).toContain('communicationMessageLabelsFromMetadata')
    expect(pageModelSource).toContain('communicationMessageSnoozeUntilFromMetadata')
    expect(surfaceSource).toContain('handleReplyAll')
    expect(surfaceSource).toContain('handleForwardMessage')
    expect(surfaceSource).toContain('handleRedirectMessage')
    expect(surfaceSource).toContain('handleMarkMessageRead')
    expect(surfaceSource).toContain('handleMarkMessageUnread')
    expect(surfaceSource).toContain('handleDeleteFromProvider')
  })
})
