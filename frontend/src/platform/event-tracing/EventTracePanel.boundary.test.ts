import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('EventTracePanel boundary', () => {
	it('preserves platform event tracing ownership after removing the legacy Vue render layer', () => {
		const appSurfaceSource = readFileSync(new URL('../../app/queries/useEventTracingViewSurface.ts', import.meta.url), 'utf8')
		const queriesSource = readFileSync(new URL('./queries.ts', import.meta.url), 'utf8')
		const typesSource = readFileSync(new URL('./types.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./EventTracePanel.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./EventTraceWorkspace.vue', import.meta.url))).toBe(false)

		expect(appSurfaceSource).toContain('Event tracing UI removed after logic extraction. Rebuild pending new design language.')
		expect(appSurfaceSource).toContain('Event tracing logic is preserved')

    expect(queriesSource).toContain('eventTraceQueryKeys')
    expect(queriesSource).toContain('useEventsQuery')
    expect(queriesSource).toContain('useEventTraceByEventIdQuery')
    expect(queriesSource).toContain('useEventTraceByCorrelationIdQuery')
    expect(queriesSource).toContain('useEventChildrenQuery')
    expect(queriesSource).toContain('fetchEvents')
    expect(queriesSource).toContain('fetchEventTraceByEventId')
    expect(queriesSource).toContain('fetchEventTraceByCorrelationId')
    expect(queriesSource).toContain('fetchEventChildren')
    expect(queriesSource).not.toContain("['telegram'")
    expect(queriesSource).not.toContain("['whatsapp'")
    expect(queriesSource).not.toContain('domains/telegram')
    expect(queriesSource).not.toContain('domains/whatsapp')

    expect(typesSource).toContain('export type EventTrace =')
    expect(typesSource).toContain('consumer_annotations')
    expect(typesSource).toContain('dead_letters')
    expect(typesSource).toContain('missing_parent_ids')
  })
})
