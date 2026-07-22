import { describe, expect, it } from 'vitest'
import { traceDataTabs, traceLookupModes } from './eventTraceSettingsPresentation'

describe('event trace settings options', () => {
  it('defines stable lookup modes and trace data tabs', () => {
    expect(traceLookupModes.map((item) => item.id)).toEqual(['event', 'trace'])
    expect(traceDataTabs.map((item) => item.id)).toEqual(['trace-events', 'recent-seeds'])
    expect(traceLookupModes.every((item) => item.icon && item.label)).toBe(true)
    expect(traceDataTabs.every((item) => item.icon && item.label)).toBe(true)
  })
})
