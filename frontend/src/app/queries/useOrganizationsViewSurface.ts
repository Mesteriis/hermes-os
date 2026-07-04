import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useOrganizationsViewSurface() {
  return createPlannedScreenSurface({
    screenId: 'organizations',
    titleKey: 'Organizations',
    descriptionKey: 'Organizations UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Organizations logic is preserved',
    detailKey: 'Organization queries and selection orchestration remain in the extracted surface. This screen stays empty until the new organizations UI is rebuilt.',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath: 'frontend/src/domains/organizations/queries/useOrganizationsPageSurface.ts'
  })
}
