import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('AgentsPage boundary', () => {
  it('preserves agents orchestration after removing the legacy AgentsPage Vue layer', () => {
    const appViewSource = readFileSync(new URL('../../../app/views/AgentsView.vue', import.meta.url), 'utf8')
    const surfaceSource = readFileSync(new URL('../queries/useAgentsPageSurface.ts', import.meta.url), 'utf8')
    const storeSource = readFileSync(new URL('../stores/agents.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./AgentsPage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/AgentsGrid.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/AgentsDetail.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/AgentsRuntimeMetrics.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/AgentsWorkflows.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/AgentsRail.vue', import.meta.url))).toBe(false)

    expect(appViewSource).toContain('Agents UI removed after logic extraction. Rebuild pending new design language.')
    expect(appViewSource).toContain('Agents logic is preserved')

    expect(surfaceSource).toContain('useAiWorkspaceQuery')
    expect(surfaceSource).toContain('useAgentsStore')
    expect(surfaceSource).toContain('store.setWorkspace')
    expect(surfaceSource).toContain('store.setLoading')
    expect(surfaceSource).toContain('refetchWorkspace')
    expect(storeSource).toContain('submitAiAnswer')
    expect(storeSource).toContain('prepareAiBrief')
    expect(storeSource).toContain('refreshTasksFromAi')
    expect(storeSource).toContain('agentCardView')
    expect(storeSource).toContain('aiModelSummary')
  })
})
