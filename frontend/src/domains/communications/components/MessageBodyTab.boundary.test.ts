import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MessageBodyTab bilingual reply boundary', () => {
  it('mounts the bilingual reply review panel without direct API access', () => {
    const source = readFileSync(
      new URL('./MessageBodyTab.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('./BilingualReplyPanel.vue')
    expect(source).toContain('isBilingualReplyOpen')
    expect(source).toContain('<BilingualReplyPanel')
    expect(source).toContain('sendBilingualReply')
    expect(source).toContain('messageId')
    expect(source).toContain('aiSummaryContractFromMetadata')
    expect(source).toContain('summaryContract')
    expect(source).toContain('ai-summary-contract')
    expect(source).toContain('Key points')
    expect(source).toContain('Action items')
    expect(source).toContain('Risks')
    expect(source).toContain('Deadlines')
    expect(source).toContain('mailExtractionSectionsFromInsight')
    expect(source).toContain('mailKnowledgeSectionsFromSummaryContract')
    expect(source).toContain('extractionSections')
    expect(source).toContain('knowledgeSections')
    expect(source).toContain('extraction-review')
    expect(source).toContain('Extraction Review')
    expect(source).toContain('knowledge-review')
    expect(source).toContain('Knowledge Review')
    expect(source).toContain('generateAiReply')
    expect(source).toContain('applyAiReply')
    expect(source).toContain('reviewSecurity')
    expect(source).toContain('reviewRecipients')
    expect(source).toContain('MessageAiReplyPanel')
    expect(source).toContain('MessageTrustReviewPanel')
    expect(source).toContain('MessageLocalIntelligencePanel')
    expect(source).toContain('remoteImageUrls')
    expect(source).toContain('shouldLoadRemoteImages')
    expect(source).toContain('remoteImageProxyUrl')
    expect(source).toContain('Remote images blocked')
    expect(source).toContain('/remote-image?url=')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
  })
})
