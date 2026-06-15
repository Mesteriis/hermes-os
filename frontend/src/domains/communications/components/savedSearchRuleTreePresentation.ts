import type { SavedSearchRuleGroup } from '../forms/savedSearchForm'

export function savedSearchRuleGroupDepthLabel(depth: number): string {
  return depth <= 0 ? 'Root group' : `Group ${depth + 1}`
}

export function savedSearchRuleGroupSummary(group: SavedSearchRuleGroup): string {
  const ruleCount = group.children.filter((child) => child.kind === 'rule').length
  const nestedGroupCount = group.children.filter((child) => child.kind === 'group').length
  const segments = [`${group.matchMode === 'all' ? 'All conditions' : 'Any condition'}`]

  if (ruleCount) segments.push(`${ruleCount} rule${ruleCount === 1 ? '' : 's'}`)
  if (nestedGroupCount) segments.push(`${nestedGroupCount} nested group${nestedGroupCount === 1 ? '' : 's'}`)
  if (!ruleCount && !nestedGroupCount) segments.push('Empty')

  return segments.join(' · ')
}
