import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('SavedSearchRuleGroupEditor boundaries', () => {
  it('renders nested group summary cues without owning query or API logic', () => {
    const source = readFileSync(new URL('./SavedSearchRuleGroupEditor.vue', import.meta.url), 'utf8')

    expect(source).toContain("from './savedSearchRuleTreePresentation'")
    expect(source).toContain('savedSearchRuleGroupDepthLabel')
    expect(source).toContain('savedSearchRuleGroupSummary')
    expect(source).toContain('saved-search-group-builder-summary')
    expect(source).toContain(':depth="nextDepth()"')
    expect(source).toContain("{{ isRoot ? 'Match' : 'Group match' }}")
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('ApiClient')
  })
})
