import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('Dialog controlled mode compatibility', () => {
  it('renders DialogTrigger only when a trigger slot is provided', () => {
    const source = readFileSync(new URL('./Dialog.vue', import.meta.url), 'utf8')

    expect(source).toContain('<DialogTrigger v-if="$slots.trigger" as-child>')
    expect(source).toContain('<DialogRoot :open="open" @update:open="(val) => emit(\'update:open\', val)">')
  })
})
