import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/knowledge/queries/useKnowledgePageSurface.ts'

export function useKnowledgeSurface() {
  return createDomainSurface({
    surfaceId: 'knowledge',
    labelKey: 'Knowledge Graph',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'knowledge-graph',
        labelKey: 'Graph',
        descriptionKey: 'Memory graph summary, selected entity state and graph projections.',
        icon: 'tabler:affiliate',
        status: 'active',
        kind: 'graph',
        contract: 'useKnowledgePageSurface.graphSummary'
      },
      {
        id: 'knowledge-search',
        labelKey: 'Semantic search',
        descriptionKey: 'Knowledge and memory search over derived, rebuildable indexes.',
        icon: 'tabler:search',
        status: 'active',
        kind: 'search',
        contract: 'useKnowledgePageSurface.searchResults'
      },
      {
        id: 'knowledge-contradictions',
        labelKey: 'Contradictions',
        descriptionKey: 'Conflicting facts and evidence review candidates.',
        icon: 'tabler:arrows-cross',
        status: 'active',
        kind: 'review',
        contract: 'useKnowledgePageSurface.contradictions'
      }
    ],
    childSurfaces: [
      {
        id: 'knowledge-graph',
        labelKey: 'Graph',
        status: 'facade',
        surfacePath,
        capabilityIds: ['knowledge-graph']
      },
      {
        id: 'knowledge-search',
        labelKey: 'Search',
        status: 'facade',
        surfacePath,
        capabilityIds: ['knowledge-search']
      },
      {
        id: 'knowledge-contradictions',
        labelKey: 'Contradictions',
        status: 'facade',
        surfacePath,
        capabilityIds: ['knowledge-contradictions']
      }
    ]
  })
}

