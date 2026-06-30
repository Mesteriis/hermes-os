import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('TasksPage boundary', () => {
  it('preserves task review orchestration after removing the TasksPage Vue layer', () => {
    const surfaceSource = readFileSync(
      new URL('../queries/useTasksPageSurface.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./TasksPage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/TaskList.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/TasksDecisionObligationReview.vue', import.meta.url))).toBe(false)
    expect(surfaceSource).toContain('useTaskCandidatesQuery')
    expect(surfaceSource).toContain('useTasksQuery')
    expect(surfaceSource).toContain('useTaskContextReviewQuery')
    expect(surfaceSource).toContain('useReviewTaskCandidateMutation')
    expect(surfaceSource).toContain('useReviewDecisionMutation')
    expect(surfaceSource).toContain('useReviewObligationMutation')
    expect(surfaceSource).toContain('reviewStats')
    expect(surfaceSource).toContain('recentCandidateSignals')
    expect(surfaceSource).toContain('setTaskCandidateReview')
    expect(surfaceSource).not.toContain("from '../api/tasks'")
  })
})
