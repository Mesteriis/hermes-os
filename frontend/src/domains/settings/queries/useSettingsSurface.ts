import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/settings/queries/useSettingsPageSurface.ts'

export function useSettingsSurface() {
  return createDomainSurface({
    surfaceId: 'settings',
    labelKey: 'Settings',
    status: 'active',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'settings-application',
        labelKey: 'Application settings',
        descriptionKey: 'Runtime flags, workspace preferences and declared settings registry.',
        icon: 'tabler:adjustments-horizontal',
        status: 'active',
        kind: 'settings',
        contract: 'useSettingsPageSurface.settingsTreeGroups'
      },
      {
        id: 'settings-accounts',
        labelKey: 'Accounts',
        descriptionKey: 'Provider accounts, capability status and sync controls.',
        icon: 'tabler:id',
        status: 'active',
        kind: 'settings',
        contract: 'useSettingsPageSurface.integrationCount'
      }
    ],
    childSurfaces: [
      {
        id: 'settings-application',
        labelKey: 'Application',
        status: 'active',
        surfacePath,
        capabilityIds: ['settings-application']
      },
      {
        id: 'settings-accounts',
        labelKey: 'Accounts',
        status: 'active',
        surfacePath,
        capabilityIds: ['settings-accounts']
      }
    ]
  })
}
