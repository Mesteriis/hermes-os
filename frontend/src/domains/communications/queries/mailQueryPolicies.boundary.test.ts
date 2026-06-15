import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('Mail query policies', () => {
  it('defines explicit background refetch policies for mail queries', () => {
    const source = readFileSync(new URL('./mailQueryPolicies.ts', import.meta.url), 'utf8')

    expect(source).toContain('mailRealtimeQueryOptions')
    expect(source).toContain('mailDetailQueryOptions')
    expect(source).toContain('mailReferenceQueryOptions')
    expect(source).toContain('refetchOnReconnect')
    expect(source).toContain('refetchOnWindowFocus')
    expect(source).toContain('refetchInterval')
  })

  it('applies shared policies from domain query hooks', () => {
    const core = readFileSync(new URL('./mailCoreQueries.ts', import.meta.url), 'utf8')
    const workspace = readFileSync(new URL('./mailWorkspaceQueries.ts', import.meta.url), 'utf8')
    const operations = readFileSync(new URL('./mailOperationQueries.ts', import.meta.url), 'utf8')

    expect(core).toContain('mailRealtimeQueryOptions')
    expect(core).toContain('mailDetailQueryOptions')
    expect(core).toContain('mailReferenceQueryOptions')
    expect(workspace).toContain('mailRealtimeQueryOptions')
    expect(workspace).toContain('mailReferenceQueryOptions')
    expect(operations).toContain('mailRealtimeQueryOptions')
  })
})
