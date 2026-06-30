import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('PersonsPage boundary', () => {
  it('preserves personas review orchestration after removing the PersonsPage Vue layer', () => {
    const surfaceSource = readFileSync(
      new URL('../queries/usePersonsPageSurface.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./PersonsPage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/PersonsList.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/PersonsDetail.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/PersonsIdentityReview.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/PersonsIdentityTraceReview.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/PersonsRelationshipReview.vue', import.meta.url))).toBe(false)

    expect(surfaceSource).toContain('usePersonsQuery')
    expect(surfaceSource).toContain('useIdentityCandidatesQuery')
    expect(surfaceSource).toContain('useIdentityTracesQuery')
    expect(surfaceSource).toContain('useRelationshipsQuery')
    expect(surfaceSource).toContain('personList = computed')
    expect(surfaceSource).toContain('selectedPerson = computed')
    expect(surfaceSource).toContain('suggestedIdentityCandidates = computed')
    expect(surfaceSource).toContain('confirmedMergeIdentityCandidates = computed')
    expect(surfaceSource).toContain('setIdentityCandidateReview')
    expect(surfaceSource).toContain('splitConfirmedIdentityMerge')
    expect(surfaceSource).toContain('selectedPersonaId = computed')
    expect(surfaceSource).not.toContain("from '../api/personas'")
  })
})
