import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/tasks/queries/useTasksPageSurface.ts'

export function useTasksSurface() {
  return createDomainSurface({
    surfaceId: 'tasks',
    labelKey: 'Tasks',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'tasks-worklist',
        labelKey: 'Task worklist',
        descriptionKey: 'Durable owner tasks, state transitions and due context.',
        icon: 'tabler:checkbox',
        status: 'active',
        kind: 'query',
        contract: 'useTasksPageSurface.tasks'
      },
      {
        id: 'tasks-candidates',
        labelKey: 'Task candidates',
        descriptionKey: 'Potential tasks extracted from communications and evidence.',
        icon: 'tabler:list-search',
        status: 'active',
        kind: 'review',
        contract: 'useTasksPageSurface.taskCandidates'
      },
      {
        id: 'tasks-obligations',
        labelKey: 'Obligations',
        descriptionKey: 'Commitments and owner obligations that may become tasks.',
        icon: 'tabler:scale',
        status: 'active',
        kind: 'evidence',
        contract: 'useTasksPageSurface.obligationCandidates'
      }
    ],
    childSurfaces: [
      {
        id: 'tasks-list',
        labelKey: 'Tasks',
        status: 'facade',
        surfacePath,
        capabilityIds: ['tasks-worklist']
      },
      {
        id: 'tasks-candidates',
        labelKey: 'Candidates',
        status: 'facade',
        surfacePath,
        capabilityIds: ['tasks-candidates']
      },
      {
        id: 'tasks-obligations',
        labelKey: 'Obligations',
        status: 'facade',
        surfacePath,
        capabilityIds: ['tasks-obligations']
      }
    ]
  })
}

