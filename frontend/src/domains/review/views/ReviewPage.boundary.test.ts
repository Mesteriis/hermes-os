import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('ReviewPage boundary', () => {
  it('preserves review orchestration after removing the ReviewPage Vue layer', () => {
    const surfaceSource = readFileSync(
      new URL('../queries/useReviewPageSurface.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./ReviewPage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/AttentionCardsPanel.vue', import.meta.url))).toBe(false)

    expect(surfaceSource).toContain('useReviewStore')
    expect(surfaceSource).toContain('promoteDrafts = ref')
    expect(surfaceSource).toContain('canonicalReviewItems = computed')
    expect(surfaceSource).toContain('suggestedRelationships = computed')
    expect(surfaceSource).toContain('suggestedDecisions = computed')
    expect(surfaceSource).toContain('suggestedObligations = computed')
    expect(surfaceSource).toContain('suggestedContradictions = computed')
    expect(surfaceSource).toContain('loadReviewWorkspace')
    expect(surfaceSource).toContain('deriveDefaultPromotion')
    expect(surfaceSource).toContain('handlePromote')
    expect(surfaceSource).toContain('reviewItemButtonPrefix')
    expect(surfaceSource).toContain('canArchive')
    expect(surfaceSource).not.toContain('../api/')
    expect(surfaceSource).not.toContain('fetch(')
  })
})
