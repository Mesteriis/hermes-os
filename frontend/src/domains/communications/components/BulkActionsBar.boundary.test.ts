import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('BulkActionsBar mail workflow boundary', () => {
  it('exposes local metadata bulk actions supported by the mail API', () => {
    const source = readFileSync(new URL('./BulkActionsBar.vue', import.meta.url), 'utf8')

    expect(source).toContain("'pin'")
    expect(source).toContain("'unpin'")
    expect(source).toContain("'important'")
    expect(source).toContain("'not_important'")
    expect(source).toContain("'add_label'")
    expect(source).toContain("'remove_label'")
    expect(source).toContain("'snooze'")
  })

  it('emits payload-backed commands for label and snooze operations', () => {
    const source = readFileSync(new URL('./BulkActionsBar.vue', import.meta.url), 'utf8')

    expect(source).toContain('type BulkActionCommand')
    expect(source).toContain('label?: string')
    expect(source).toContain('snooze_until?: string')
    expect(source).toContain("label: 'Follow up'")
    expect(source).toContain('snooze_until: nextBusinessMorningIso()')
  })
})
