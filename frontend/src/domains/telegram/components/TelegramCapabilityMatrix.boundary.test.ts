import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramCapabilityMatrix boundary', () => {
  it('loads account-scoped capabilities through the query layer', () => {
    const source = readFileSync(new URL('./TelegramCapabilityMatrix.vue', import.meta.url), 'utf8')

    expect(source).toContain('useTelegramAccountCapabilitiesQuery')
    expect(source).toContain('unsupported_features')
    expect(source).toContain('capability.operation')
    expect(source).toContain('capability.status')
    expect(source).toContain("t('Capabilities')")
    expect(source).toContain('confirmation_required')
    expect(source).toContain('closure_gate')
    expect(source).not.toContain('fetch(')
  })
})
