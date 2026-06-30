import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('KnowledgePage boundary', () => {
  it('preserves knowledge graph orchestration after removing the KnowledgePage Vue layer', () => {
    const surfaceSource = readFileSync(
      new URL('../queries/useKnowledgePageSurface.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./KnowledgePage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/KnowledgeGraphCanvas.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/KnowledgeNodeInspector.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/KnowledgePolygraphReview.vue', import.meta.url))).toBe(false)

    expect(surfaceSource).toContain('useKnowledgeStore')
    expect(surfaceSource).toContain('useGraphSummaryQuery')
    expect(surfaceSource).toContain('useContradictionsQuery')
    expect(surfaceSource).toContain('watch(summaryData')
    expect(surfaceSource).toContain('watch(summaryError')
    expect(surfaceSource).toContain('watch(contradictionsData')
    expect(surfaceSource).toContain('handleSearch')
    expect(surfaceSource).toContain('handleSelectSearchResult')
    expect(surfaceSource).toContain('suggestedContradictionsCount = computed')
    expect(surfaceSource).toContain('loadGraphNodeChoices')
    expect(surfaceSource).not.toContain('../api/')
    expect(surfaceSource).not.toContain('fetch(')
  })
})
