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
        id: 'settings-appearance',
        labelKey: 'Appearance',
        descriptionKey: 'Theme, density, language and navigation preferences.',
        icon: 'tabler:palette',
        status: 'active',
        kind: 'settings',
        contract: 'useSettingsPageSurface.theme'
      },
      {
        id: 'settings-integrations',
        labelKey: 'Integrations',
        descriptionKey: 'Provider accounts, capability status and sync controls.',
        icon: 'tabler:plug-connected',
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
        id: 'settings-interface',
        labelKey: 'Interface',
        status: 'active',
        surfacePath,
        capabilityIds: ['settings-appearance']
      },
      {
        id: 'settings-sources',
        labelKey: 'Sources',
        status: 'active',
        surfacePath,
        capabilityIds: ['settings-integrations']
      }
    ]
  })
}

