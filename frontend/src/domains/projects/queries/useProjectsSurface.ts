import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/projects/queries/useProjectsPageSurface.ts'

export function useProjectsSurface() {
  return createDomainSurface({
    surfaceId: 'projects',
    labelKey: 'Projects',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'projects-list',
        labelKey: 'Project list',
        descriptionKey: 'Owner projects with status, context and next-step summaries.',
        icon: 'tabler:briefcase',
        status: 'active',
        kind: 'query',
        contract: 'useProjectsPageSurface.projects'
      },
      {
        id: 'projects-detail',
        labelKey: 'Project detail',
        descriptionKey: 'Selected project facts, evidence and related communications context.',
        icon: 'tabler:layout-list-detail',
        status: 'active',
        kind: 'inspector',
        contract: 'useProjectsPageSurface.selectedProject'
      },
      {
        id: 'projects-candidates',
        labelKey: 'Project candidates',
        descriptionKey: 'Potential project records waiting for review-backed promotion.',
        icon: 'tabler:sparkles',
        status: 'active',
        kind: 'review',
        contract: 'useProjectsPageSurface.projectCandidates'
      }
    ],
    childSurfaces: [
      {
        id: 'projects-overview',
        labelKey: 'Overview',
        status: 'facade',
        surfacePath,
        capabilityIds: ['projects-list']
      },
      {
        id: 'projects-inspector',
        labelKey: 'Inspector',
        status: 'facade',
        surfacePath,
        capabilityIds: ['projects-detail']
      },
      {
        id: 'projects-review',
        labelKey: 'Review',
        status: 'facade',
        surfacePath,
        capabilityIds: ['projects-candidates']
      }
    ]
  })
}

