import { describe, expect, it } from 'vitest'
import { filterBackgroundJobRows } from './backgroundJobsPresentation'
import type { BackgroundJobRow } from './backgroundJobsPresentation'

describe('background jobs presentation', () => {
  it('filters rows by group while preserving all rows for the all filter', () => {
    const rows = [row('mail'), row('ai')]
    expect(filterBackgroundJobRows(rows, 'mail')).toEqual([rows[0]])
    expect(filterBackgroundJobRows(rows, 'all')).toBe(rows)
  })
})

function row(group: BackgroundJobRow['group']): BackgroundJobRow {
  return {
    id: group,
    group,
    label: group,
    description: group,
    icon: 'tabler:clock',
    sourceCode: null,
    runtimeKinds: [],
    cadence: '',
    evidence: '',
    controlSection: null,
    groupLabel: group,
    statusLabel: 'Ready',
    statusDetail: '',
    tone: 'good',
    metric: '',
    lastActivityLabel: '',
    nextRunLabel: '',
    observedRuntimeCount: 0
  }
}
