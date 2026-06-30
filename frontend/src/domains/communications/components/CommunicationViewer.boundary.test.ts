import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('CommunicationViewer boundary', () => {
  it('preserves AI state query and transition contracts after removing the viewer render layer', () => {
    const coreQuerySource = readFileSync(
      new URL('../queries/mailCoreQueries.ts', import.meta.url),
      'utf8'
    )
    const operationQuerySource = readFileSync(
      new URL('../queries/mailOperationQueries.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./CommunicationViewer.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./MessageHeadersTab.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./MessageTimelineTab.vue', import.meta.url))).toBe(false)
    expect(coreQuerySource).toContain('export function useMessageAiStateQuery')
    expect(operationQuerySource).toContain('export function useUpdateMessageAiStateMutation')
    expect(operationQuerySource).toContain("queryClient.setQueryData(['communications-ai-state', record.message_id], record)")
    expect(operationQuerySource).toContain("queryClient.invalidateQueries({ queryKey: ['communications-ai-state', record.message_id] })")
    expect(operationQuerySource).toContain("queryClient.invalidateQueries({ queryKey: ['communications-message', record.message_id] })")
  })
})
