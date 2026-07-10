import { computed, ref } from 'vue'
import { useReviewStore } from '../stores/review'
import type {
  Decision,
  Obligation,
  Relationship,
  ReviewItem,
  ReviewItemPromotionRequest,
  ReviewWorkspaceItemAction
} from '../types/review'

export function useReviewPageSurface() {
  const store = useReviewStore()
  const promoteDrafts = ref<Record<string, ReviewItemPromotionRequest>>({})

  const canonicalReviewItems = computed(() =>
    store.reviewItems.filter((item) => item.status === 'new' || item.status === 'in_review')
  )
  const suggestedRelationships = computed(() =>
    store.relationships.filter((item) => item.review_state === 'suggested')
  )
  const suggestedDecisions = computed(() =>
    store.decisions.filter((item) => item.review_state === 'suggested')
  )
  const suggestedObligations = computed(() =>
    store.obligations.filter((item) => item.review_state === 'suggested')
  )
  const suggestedContradictions = computed(() =>
    store.contradictions.filter((item) => item.review_state === 'suggested')
  )
  const attentionCards = computed(() => store.attentionCards)

  async function loadReviewWorkspace() {
    await store.loadAll()
    syncPromoteDrafts()
  }

  function relationshipPeer(item: Relationship): string {
    return `${item.source_entity_kind}:${item.source_entity_id} -> ${item.target_entity_kind}:${item.target_entity_id}`
  }

  function decisionEntityLabel(item: Decision): string {
    if (!item.decided_by_entity_kind || !item.decided_by_entity_id) return 'Unknown'
    return `${item.decided_by_entity_kind}:${item.decided_by_entity_id}`
  }

  function obligationEntityLabel(item: Obligation): string {
    return `${item.obligated_entity_kind}:${item.obligated_entity_id}`
  }

  function formatItemTime(value: string | null | undefined): string {
    if (!value) return ''
    const date = new Date(value)
    if (Number.isNaN(date.getTime())) return ''
    return new Intl.DateTimeFormat('en', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    }).format(date)
  }

  async function handleReview(action: ReviewWorkspaceItemAction) {
    await store.reviewItem(action)
  }

  function syncPromoteDrafts() {
    store.reviewItems.forEach((item: ReviewItem) => {
      if (promoteDrafts.value[item.review_item_id]) return
      promoteDrafts.value[item.review_item_id] = deriveDefaultPromotion(item)
    })
  }

  function deriveDefaultPromotion(item: ReviewItem): ReviewItemPromotionRequest {
    const defaults: Record<string, ReviewItemPromotionRequest> = {
      new_persona: { target_domain: 'personas', target_entity_kind: 'persona', target_entity_id: '' },
      new_organization: {
        target_domain: 'organizations',
        target_entity_kind: 'organization',
        target_entity_id: ''
      },
      potential_task: { target_domain: 'tasks', target_entity_kind: 'task', target_entity_id: '' },
      potential_obligation: {
        target_domain: 'obligations',
        target_entity_kind: 'obligation',
        target_entity_id: ''
      },
      potential_decision: {
        target_domain: 'decisions',
        target_entity_kind: 'decision',
        target_entity_id: ''
      },
      potential_relationship: {
        target_domain: 'relationships',
        target_entity_kind: 'relationship',
        target_entity_id: ''
      },
      potential_project: {
        target_domain: 'projects',
        target_entity_kind: 'project',
        target_entity_id: ''
      },
      knowledge_candidate: {
        target_domain: 'documents',
        target_entity_kind: 'document',
        target_entity_id: `document:review-note:${item.review_item_id}`
      }
    }
    return defaults[item.item_kind] ?? { target_domain: '', target_entity_kind: '', target_entity_id: '' }
  }

  function canPromote(item: ReviewItem): boolean {
    const draft = promoteDrafts.value[item.review_item_id]
    return !!(
      draft &&
      draft.target_domain.trim() &&
      draft.target_entity_kind.trim() &&
      draft.target_entity_id.trim() &&
      store.reviewingItemKey !== `review_item_promote:${item.review_item_id}`
    )
  }

  async function handlePromote(item: ReviewItem) {
    const draft = promoteDrafts.value[item.review_item_id]
    if (!draft) return
    await handleReview({
      kind: 'review_item_promote',
      item,
      promotion: { ...draft }
    })
  }

  function reviewItemButtonPrefix(item: ReviewItem): string {
    return `review_item:${item.review_item_id}`
  }

  function canArchive(item: ReviewItem): boolean {
    return store.reviewingItemKey !== `review_item_archive:${item.review_item_id}`
  }

  function reviewItemKindLabel(itemKind: ReviewItem['item_kind']): string {
    return itemKind
  }

  return {
    attentionCards,
    canonicalReviewItems,
    canArchive,
    canPromote,
    decisionEntityLabel,
    formatItemTime,
    handlePromote,
    handleReview,
    loadReviewWorkspace,
    obligationEntityLabel,
    promoteDrafts,
    relationshipPeer,
    reviewItemButtonPrefix,
    reviewItemKindLabel,
    store,
    suggestedContradictions,
    suggestedDecisions,
    suggestedObligations,
    suggestedRelationships,
    syncPromoteDrafts,
    deriveDefaultPromotion
  }
}
