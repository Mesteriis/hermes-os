import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/agents/queries/useAgentsPageSurface.ts'

export function useAgentsSurface() {
  return createDomainSurface({
    surfaceId: 'agents',
    labelKey: 'AI Agents',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'agents-workspace',
        labelKey: 'Agent workspace',
        descriptionKey: 'Local AI workspace orchestration and readiness state.',
        icon: 'tabler:sparkles',
        status: 'active',
        kind: 'workspace',
        contract: 'useAgentsPageSurface.workspaceData'
      },
      {
        id: 'agents-runs',
        labelKey: 'Runs',
        descriptionKey: 'Agent run review, action outputs and owner-controlled execution history.',
        icon: 'tabler:player-play',
        status: 'active',
        kind: 'timeline',
        contract: 'useAgentsPageSurface.store'
      },
      {
        id: 'agents-actions',
        labelKey: 'Suggested actions',
        descriptionKey: 'AI-proposed actions that remain candidates until review.',
        icon: 'tabler:wand',
        status: 'active',
        kind: 'review',
        contract: 'useAgentsPageSurface.workspaceData'
      }
    ],
    childSurfaces: [
      {
        id: 'agents-workspace',
        labelKey: 'Workspace',
        status: 'facade',
        surfacePath,
        capabilityIds: ['agents-workspace']
      },
      {
        id: 'agents-runs',
        labelKey: 'Runs',
        status: 'facade',
        surfacePath,
        capabilityIds: ['agents-runs']
      },
      {
        id: 'agents-actions',
        labelKey: 'Actions',
        status: 'facade',
        surfacePath,
        capabilityIds: ['agents-actions']
      }
    ]
  })
}

