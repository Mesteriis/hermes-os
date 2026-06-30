import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('OrganizationsPage boundary', () => {
  it('preserves organizations selection orchestration after removing the legacy OrganizationsPage Vue layer', () => {
    const appViewSource = readFileSync(new URL('../../../app/views/OrganizationsView.vue', import.meta.url), 'utf8')
    const surfaceSource = readFileSync(new URL('../queries/useOrganizationsPageSurface.ts', import.meta.url), 'utf8')
    const querySource = readFileSync(new URL('../queries/useOrganizationsQuery.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./OrganizationsPage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/OrganizationsList.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/OrganizationsDetail.vue', import.meta.url))).toBe(false)

    expect(appViewSource).toContain('Organizations UI removed after logic extraction. Rebuild pending new design language.')
    expect(appViewSource).toContain('Organizations logic is preserved')

    expect(surfaceSource).toContain('useOrganizationsQuery')
    expect(surfaceSource).toContain('selectedOrganizationId')
    expect(surfaceSource).toContain('selectedOrganization')
    expect(surfaceSource).toContain('selectOrganization')
    expect(surfaceSource).toContain('orgPeople')
    expect(querySource).toContain('useOrganizationsQuery')
    expect(querySource).toContain('useOrganizationQuery')
  })
})
