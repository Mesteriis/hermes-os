import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('Communication query policies', () => {
  it('defines explicit background refetch policies for communication queries', () => {
    const source = readFileSync(new URL('./communicationQueryPolicies.ts', import.meta.url), 'utf8')

    expect(source).toContain('communicationRealtimeQueryOptions')
    expect(source).toContain('communicationDetailQueryOptions')
    expect(source).toContain('communicationReferenceQueryOptions')
    expect(source).toContain('refetchOnReconnect')
    expect(source).toContain('refetchOnWindowFocus')
    expect(source).toContain('refetchInterval')
  })

  it('applies shared policies from domain query hooks', () => {
    const core = readFileSync(new URL('./mailCoreQueries.ts', import.meta.url), 'utf8')
    const workspace = readFileSync(new URL('./mailWorkspaceQueries.ts', import.meta.url), 'utf8')
    const operations = readFileSync(new URL('./mailOperationQueries.ts', import.meta.url), 'utf8')

    expect(core).toContain('communicationRealtimeQueryOptions')
    expect(core).toContain('communicationDetailQueryOptions')
    expect(core).toContain('communicationReferenceQueryOptions')
    expect(workspace).toContain('communicationRealtimeQueryOptions')
    expect(workspace).toContain('communicationReferenceQueryOptions')
    expect(operations).toContain('communicationRealtimeQueryOptions')
  })
})
