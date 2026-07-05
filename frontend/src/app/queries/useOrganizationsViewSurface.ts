import { useOrganizationsSurface } from '../../domains/organizations/queries/useOrganizationsSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useOrganizationsViewSurface() {
  const organizations = useOrganizationsSurface()

  return createPlannedScreenSurface({
    screenId: 'organizations',
    titleKey: 'Organizations',
    descriptionKey: 'Organizations UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Organizations logic is preserved',
    detailKey: 'Organization queries and selection orchestration remain in the extracted surface. This screen stays empty until the new organizations UI is rebuilt.',
    status: organizations.status,
    ownerLayer: 'domain',
    surfacePath: organizations.surfacePath,
    childSurfaces: organizations.childSurfaces
  })
}
