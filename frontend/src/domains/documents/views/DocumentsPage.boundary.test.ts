import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('DocumentsPage boundary', () => {
  it('preserves documents orchestration after removing the DocumentsPage Vue layer', () => {
    const surfaceSource = readFileSync(
      new URL('../queries/useDocumentsPageSurface.ts', import.meta.url),
      'utf8'
    )
    const storeSource = readFileSync(
      new URL('../stores/documents.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./DocumentsPage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/DocumentsInsights.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/DocumentsList.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/DocumentsNavigation.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/DocumentsProcessingJobs.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/DocumentsSourceCards.vue', import.meta.url))).toBe(false)
    expect(surfaceSource).toContain('useDocumentProcessingJobsQuery')
    expect(surfaceSource).toContain('useRetryDocumentProcessingJobMutation')
    expect(surfaceSource).toContain('handleRetry')
    expect(surfaceSource).toContain('documentProcessingJobs')
    expect(surfaceSource).toContain('documents')
    expect(surfaceSource).not.toContain('../api/documents')
    expect(surfaceSource).not.toContain('retryDocumentProcessingJob')
    expect(surfaceSource).not.toContain('globalThis.fetch')
    expect(surfaceSource).not.toContain('window.fetch')
    expect(storeSource).toContain('setSearchQuery')
    expect(storeSource).toContain('setActiveFilter')
    expect(storeSource).toContain('setDocumentsError')
    expect(storeSource).toContain('setRetryingJobId')
  })
})
