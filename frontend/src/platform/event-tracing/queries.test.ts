import { describe, expect, it } from 'vitest'
import { eventTraceQueryKeys } from './queries'

describe('event trace query keys', () => {
  it('uses provider-neutral event tracing keys', () => {
    expect(eventTraceQueryKeys.events(0, 50)).toEqual(['events', 'list', 0, 50])
    expect(eventTraceQueryKeys.byEvent('event-1')).toEqual(['events', 'event-1', 'trace'])
    expect(eventTraceQueryKeys.byCorrelation('trace-1')).toEqual(['event-traces', 'trace-1'])
    expect(eventTraceQueryKeys.children('event-1')).toEqual(['events', 'event-1', 'children'])

    const flattened = [
      ...eventTraceQueryKeys.events(0, 50),
      ...eventTraceQueryKeys.byEvent('event-1'),
      ...eventTraceQueryKeys.byCorrelation('trace-1'),
      ...eventTraceQueryKeys.children('event-1')
    ].join(' ')
    expect(flattened).not.toContain('telegram')
    expect(flattened).not.toContain('whatsapp')
  })
})
