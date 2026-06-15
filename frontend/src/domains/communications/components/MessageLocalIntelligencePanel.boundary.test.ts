import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MessageLocalIntelligencePanel boundary', () => {
  it('runs explain and language detection through query hooks without direct API access', () => {
    const source = readFileSync(new URL('./MessageLocalIntelligencePanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('useExplainMessageMutation')
    expect(source).toContain('useDetectMessageLanguageMutation')
    expect(source).toContain('Importance')
    expect(source).toContain('Detect language')
    expect(source).toContain('Why this matters')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
  })

})
