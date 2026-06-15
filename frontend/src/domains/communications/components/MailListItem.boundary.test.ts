import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MailListItem drag payload boundary', () => {
  it('serializes the full selected message set when dragging a selected row', () => {
    const source = readFileSync(new URL('./MailListItem.vue', import.meta.url), 'utf8')

    expect(source).toContain('selectedMessageIds: string[]')
    expect(source).toContain('createMailMessageDragPayload(props.message.message_id, props.selectedMessageIds)')
    expect(source).toContain('role="option"')
    expect(source).toContain(':aria-selected="isChecked || isSelected"')
  })
})
