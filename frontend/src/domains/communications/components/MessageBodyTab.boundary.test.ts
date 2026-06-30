import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('MessageBodyTab bilingual reply boundary', () => {
  it('preserves bilingual reply, summary-contract, and local intelligence helpers after removing the message body render layer', () => {
    const pageModelSource = readFileSync(
      new URL('../helpers/communicationPageModels.ts', import.meta.url),
      'utf8'
    )
    const actionQuerySource = readFileSync(
      new URL('../queries/mailActionQueries.ts', import.meta.url),
      'utf8'
    )
    const operationQuerySource = readFileSync(
      new URL('../queries/mailOperationQueries.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./MessageBodyTab.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./BilingualReplyPanel.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./MessageAiReplyPanel.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./MessageLocalIntelligencePanel.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./MessageTrustReviewPanel.vue', import.meta.url))).toBe(false)
    expect(pageModelSource).toContain('aiSummaryContractFromMetadata')
    expect(pageModelSource).toContain('communicationExtractionSectionsFromInsight')
    expect(pageModelSource).toContain('communicationKnowledgeSectionsFromSummaryContract')
    expect(actionQuerySource).toContain('export function useGenerateAiReplyVariantsMutation()')
    expect(actionQuerySource).toContain('export function useExplainMessageMutation()')
    expect(actionQuerySource).toContain('export function useDetectMessageLanguageMutation()')
    expect(operationQuerySource).toContain('export function usePrepareBilingualReplyFlowMutation()')
  })
})
