import { computed } from 'vue'
import {
  useMailBlockersQuery,
  useSubscriptionsQuery,
  useTopSendersQuery
} from '../queries/mailWorkspaceQueries'
import type { QueryParam } from '../queries/queryTypes'

export function useMailResourceOverview(accountId?: QueryParam<string>) {
  const {
    data: subscriptionsData,
    isLoading: isSubscriptionsLoading,
    hasNextPage: hasSubscriptionsNextPage,
    isFetchingNextPage: isFetchingSubscriptionsNextPage,
    fetchNextPage: fetchNextSubscriptionsPage
  } = useSubscriptionsQuery(accountId)
  const {
    data: topSendersData,
    isLoading: isTopSendersLoading,
    hasNextPage: hasTopSendersNextPage,
    isFetchingNextPage: isFetchingTopSendersNextPage,
    fetchNextPage: fetchNextTopSendersPage
  } = useTopSendersQuery(accountId)
  const {
    data: blockersData,
    isLoading: isBlockersLoading
  } = useMailBlockersQuery()

  const subscriptions = computed(() => subscriptionsData.value ?? [])
  const topSenders = computed(() => topSendersData.value ?? [])
  const blockers = computed(() => blockersData.value ?? [])
  const hasMoreSubscriptions = computed(() => Boolean(hasSubscriptionsNextPage.value))
  const hasMoreTopSenders = computed(() => Boolean(hasTopSendersNextPage.value))
  const isLoadingMoreSubscriptions = computed(() => isFetchingSubscriptionsNextPage.value)
  const isLoadingMoreTopSenders = computed(() => isFetchingTopSendersNextPage.value)
  const areResourcesLoading = computed(() =>
    isSubscriptionsLoading.value || isTopSendersLoading.value || isBlockersLoading.value
  )

  function handleLoadMoreSubscriptions() {
    if (!hasSubscriptionsNextPage.value || isFetchingSubscriptionsNextPage.value) return
    void fetchNextSubscriptionsPage()
  }

  function handleLoadMoreTopSenders() {
    if (!hasTopSendersNextPage.value || isFetchingTopSendersNextPage.value) return
    void fetchNextTopSendersPage()
  }

  return {
    areResourcesLoading,
    blockers,
    handleLoadMoreSubscriptions,
    handleLoadMoreTopSenders,
    hasMoreSubscriptions,
    hasMoreTopSenders,
    isLoadingMoreSubscriptions,
    isLoadingMoreTopSenders,
    subscriptions,
    topSenders
  }
}
