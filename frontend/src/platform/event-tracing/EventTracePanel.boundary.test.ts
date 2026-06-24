import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('EventTracePanel boundary', () => {
  it('stays in platform event tracing ownership', () => {
    const source = readFileSync(new URL('./EventTracePanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('EventTrace')
    expect(source).toContain('consumer_annotations')
    expect(source).toContain('dead_letters')
    expect(source).toContain('missing_parent_ids')
    expect(source).not.toContain("['telegram'")
    expect(source).not.toContain("['whatsapp'")
    expect(source).not.toContain('domains/telegram')
    expect(source).not.toContain('domains/whatsapp')
  })
})
