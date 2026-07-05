import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/review/queries/useReviewPageSurface.ts'

export function useReviewSurface() {
  return createDomainSurface({
    surfaceId: 'review',
    labelKey: 'Review',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'review-inbox',
        labelKey: 'Review inbox',
        descriptionKey: 'Observed candidates waiting for owner review.',
        icon: 'tabler:clipboard-check',
        status: 'active',
        kind: 'review',
        contract: 'useReviewPageSurface.canonicalReviewItems'
      },
      {
        id: 'review-promotion',
        labelKey: 'Promotion',
        descriptionKey: 'Evidence-backed promotion into durable domain entities.',
        icon: 'tabler:arrow-up-right',
        status: 'active',
        kind: 'command',
        contract: 'useReviewPageSurface.handlePromote'
      },
      {
        id: 'review-attention',
        labelKey: 'Attention cards',
        descriptionKey: 'Cross-domain review pressure and unresolved candidate groups.',
        icon: 'tabler:alert-triangle',
        status: 'active',
        kind: 'projection',
        contract: 'useReviewPageSurface.attentionCards'
      }
    ],
    childSurfaces: [
      {
        id: 'review-inbox',
        labelKey: 'Inbox',
        status: 'facade',
        surfacePath,
        capabilityIds: ['review-inbox']
      },
      {
        id: 'review-promotion',
        labelKey: 'Promotion',
        status: 'facade',
        surfacePath,
        capabilityIds: ['review-promotion']
      },
      {
        id: 'review-attention',
        labelKey: 'Attention',
        status: 'facade',
        surfacePath,
        capabilityIds: ['review-attention']
      }
    ]
  })
}

