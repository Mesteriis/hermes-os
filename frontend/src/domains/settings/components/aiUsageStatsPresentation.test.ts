import { describe, expect, it } from 'vitest'
import { maxHourlyRequestCount } from './aiUsageStatsPresentation'

describe('ai usage stats presentation', () => {
  it('keeps a non-zero chart scale for empty and zero-count buckets', () => {
    expect(maxHourlyRequestCount([])).toBe(1)
    expect(maxHourlyRequestCount([
      { hour: '2026-07-21T10', label: '10', requestCount: 0, failedCount: 0, estimatedTokens: 0 },
      { hour: '2026-07-21T11', label: '11', requestCount: 4, failedCount: 1, estimatedTokens: 20 },
    ])).toBe(4)
  })
})
