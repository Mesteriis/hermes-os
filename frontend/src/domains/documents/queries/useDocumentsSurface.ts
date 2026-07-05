import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/documents/queries/useDocumentsPageSurface.ts'

export function useDocumentsSurface() {
  return createDomainSurface({
    surfaceId: 'documents',
    labelKey: 'Documents',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'documents-library',
        labelKey: 'Document library',
        descriptionKey: 'Document metadata, source references and processing state.',
        icon: 'tabler:file-text',
        status: 'active',
        kind: 'query',
        contract: 'useDocumentsPageSurface.documents'
      },
      {
        id: 'documents-processing',
        labelKey: 'Processing jobs',
        descriptionKey: 'Document ingestion, retry and review workflow state.',
        icon: 'tabler:loader',
        status: 'active',
        kind: 'automation',
        contract: 'useDocumentsPageSurface.processingJobs'
      },
      {
        id: 'documents-evidence',
        labelKey: 'Evidence',
        descriptionKey: 'Document-backed evidence paths consumed by review and memory.',
        icon: 'tabler:shield-check',
        status: 'active',
        kind: 'evidence',
        contract: 'useDocumentsPageSurface.evidenceCards'
      }
    ],
    childSurfaces: [
      {
        id: 'documents-library',
        labelKey: 'Library',
        status: 'facade',
        surfacePath,
        capabilityIds: ['documents-library']
      },
      {
        id: 'documents-processing',
        labelKey: 'Processing',
        status: 'facade',
        surfacePath,
        capabilityIds: ['documents-processing']
      },
      {
        id: 'documents-evidence',
        labelKey: 'Evidence',
        status: 'facade',
        surfacePath,
        capabilityIds: ['documents-evidence']
      }
    ]
  })
}

