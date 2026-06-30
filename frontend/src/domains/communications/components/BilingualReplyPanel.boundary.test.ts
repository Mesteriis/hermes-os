import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('BilingualReplyPanel boundary', () => {
  it('preserves the bilingual reply form schema and mutation contract after removing the render layer', () => {
    const formSource = readFileSync(
      new URL('../forms/bilingualReplyFlowForm.ts', import.meta.url),
      'utf8'
    )
    const operationQuerySource = readFileSync(
      new URL('../queries/mailOperationQueries.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./BilingualReplyPanel.vue', import.meta.url))).toBe(false)
    expect(formSource).toContain("from '@vee-validate/zod'")
    expect(formSource).toContain('bilingualReplyFlowFormSchema')
    expect(formSource).toContain('bilingualReplyFlowFormDefaults')
    expect(formSource).toContain('bilingualReplyFlowFormToRequest')
    expect(operationQuerySource).toContain('export function usePrepareBilingualReplyFlowMutation()')
  })
})
