import { describe, expect, it } from 'vitest'
import {
  canAdvanceIntegrationConnectionWizard,
  integrationCheckIcon,
  integrationProviderIconTone,
} from './integrationConnectionWizardPresentation'

describe('integration connection wizard presentation', () => {
  it('applies provider step advancement and status icon policy', () => {
    expect(canAdvanceIntegrationConnectionWizard(1, false, 'mail', false)).toBe(false)
    expect(canAdvanceIntegrationConnectionWizard(1, true, 'mail', false)).toBe(true)
    expect(canAdvanceIntegrationConnectionWizard(2, false, 'telegram', false)).toBe(false)
    expect(canAdvanceIntegrationConnectionWizard(2, false, 'telegram', true)).toBe(true)
    expect(integrationProviderIconTone('whatsapp')).toContain('whatsapp')
    expect(integrationCheckIcon('blocked')).toBe('tabler:alert-triangle')
  })
})
