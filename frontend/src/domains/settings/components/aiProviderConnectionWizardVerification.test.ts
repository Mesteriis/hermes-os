import { describe, expect, it } from 'vitest'
import { aiProviderWizardVerificationSucceeded } from './aiProviderConnectionWizardPresentation'

describe('AI provider connection wizard verification', () => {
  it('allows the wizard to continue only after successful verification', () => {
    expect(aiProviderWizardVerificationSucceeded('ok')).toBe(true)
    expect(aiProviderWizardVerificationSucceeded('error')).toBe(false)
    expect(aiProviderWizardVerificationSucceeded('pending')).toBe(false)
  })
})
