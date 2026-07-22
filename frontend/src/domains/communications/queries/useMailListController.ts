import { computed, ref, watch } from 'vue'
import { useI18n } from '@/platform/i18n'
import type { TreeSelectOption } from '@/shared/ui'
import {
  createMailListSearchBuilderState,
  mailListSearchBuilderAddClause,
  mailListSearchBuilderActiveFieldGroup,
  mailListSearchBuilderCanAdd,
  mailListSearchBuilderCanApply,
  mailListSearchBuilderCanSave,
  mailListSearchBuilderClauseViews,
  mailListSearchBuilderClear,
  mailListSearchBuilderOperatorItems,
  mailListSearchBuilderPresetItems,
  mailListSearchBuilderQuery,
  mailListSearchBuilderRemoveClause,
  mailListSearchBuilderSetField,
  mailListSearchBuilderSetMatchMode,
  mailListSearchBuilderSetOperator,
  mailListSearchBuilderSetValue,
  mailListSearchLocalizedToggleItems,
  mailListSearchFieldGroups,
  mailListSearchFieldItem,
  mailListSearchMatchModeItems,
} from '../components/mail/mailSearchBuilder'
import { mailListSearchBuilderValueSuggestions } from '../components/mail/mailSearchSuggestions'
import {
  committedSearchBuilderState,
  cloneSearchBuilderState,
  createSavedFilter,
  findSavedFilter,
  savedFilterTreeOptions,
} from '../components/mail/mailListSavedFilters'
import type { MailListItemDensity, MailListItemModel } from '../components/mail/mailElements'
import {
  isMailListViewId,
  mailListItemIds,
  mailListItemsForView,
  mailListTreeSelectOptions,
  type MailListViewId,
} from '../components/mail/mailListViews'
import { mailListItemsForSearch } from '../components/mail/mailSearchBuilder'
import type { MailListSavedFilter } from '../components/mail/mailListSavedFilters'

export interface MailListControllerProps {
  items: readonly MailListItemModel[]
  hasMoreItems?: boolean
  isLoadingMore?: boolean
  searchQuery?: string
}

export interface MailListControllerActions {
  loadMore: () => void
  refresh: () => void
  importMailFile: (file: File) => void
  selectItem: (item: MailListItemModel) => void
  updateSearchQuery: (query: string) => void
  visibleItemsChange: (itemIds: string[]) => void
}

export function useMailListController(
  props: Readonly<MailListControllerProps>,
  actions: MailListControllerActions,
) {
  const { t } = useI18n()
  const loadMoreScrollThresholdPx = 320

  const activeDensity = ref<MailListItemDensity>('comfortable')
  const activeMailViewId = ref<MailListViewId | string>('mail:all')
  const activeSearchBuilderGroupId = ref(mailListSearchFieldGroups[0]?.id ?? 'text')
  const isSearchBuilderOpen = ref(false)
  const isPlainSearchOpen = ref(Boolean(props.searchQuery?.trim()))
  const syncProgressVisible = ref(false)
  const searchBuilderState = ref(createMailListSearchBuilderState())
  const savedFilterName = ref('')
  const savedFilters = ref<MailListSavedFilter[]>([])
  const nextSavedFilterId = ref(1)

  const plainSearchQuery = computed(() => props.searchQuery ?? '')
  const plainSearchIsActive = computed(() => plainSearchQuery.value.trim().length > 0)
  const plainSearchButtonClass = computed(() => {
    const base = 'mail-list-view-select-row__search hermes-icon-button'
    return plainSearchIsActive.value ? `${base} mail-list-view-select-row__search--active` : base
  })

  const builderSearchQuery = computed(() => mailListSearchBuilderQuery(searchBuilderState.value))
  const listItems = computed(() => props.items)
  const viewItems = computed(() => mailListItemsForView(listItems.value, activeMailViewId.value))
  const visibleItems = computed(() => mailListItemsForSearch(viewItems.value, builderSearchQuery.value))

  const mailViewOptions = computed<TreeSelectOption[]>(() => {
    return mailListTreeSelectOptions(
      listItems.value,
      savedFilterTreeOptions(savedFilters.value, t),
      t,
      Boolean(props.hasMoreItems),
    )
  })

  const searchBuilderCanAdd = computed(() => mailListSearchBuilderCanAdd(searchBuilderState.value))
  const searchBuilderCanApply = computed(() => mailListSearchBuilderCanApply(searchBuilderState.value))
  const searchBuilderCanSave = computed(() => mailListSearchBuilderCanSave(
    searchBuilderState.value,
    savedFilterName.value,
  ))
  const searchBuilderClauseViews = computed(() => mailListSearchBuilderClauseViews(searchBuilderState.value))
  const searchBuilderFieldGroups = computed(() => mailListSearchFieldGroups)
  const activeSearchBuilderFieldGroup = computed(() => {
    return mailListSearchBuilderActiveFieldGroup(activeSearchBuilderGroupId.value)
  })
  const localizedSearchMatchModeItems = computed(() => mailListSearchLocalizedToggleItems(mailListSearchMatchModeItems, t))
  const searchBuilderOperatorOptions = computed(() => {
    return mailListSearchLocalizedToggleItems(mailListSearchBuilderOperatorItems(searchBuilderState.value), t)
  })
  const searchBuilderPresetOptions = computed(() => {
    return mailListSearchLocalizedToggleItems(mailListSearchBuilderPresetItems(searchBuilderState.value), t)
  })
  const searchBuilderValuePlaceholder = computed(() => t(mailListSearchFieldItem(searchBuilderState.value.field).placeholder))
  const searchBuilderValueSuggestions = computed(() => {
    return mailListSearchBuilderValueSuggestions(viewItems.value, searchBuilderState.value)
  })

  watch(
    visibleItems,
    (items) => {
      actions.visibleItemsChange(mailListItemIds(items))
    },
    { immediate: true },
  )

  watch(
    () => props.searchQuery,
    (query) => {
      if (query?.trim()) isPlainSearchOpen.value = true
    },
  )

  function selectDensity(value: MailListItemDensity): void {
    activeDensity.value = value
  }

  function densityIsActive(value: MailListItemDensity): boolean {
    return activeDensity.value === value
  }

  function densityMenuItemClass(value: MailListItemDensity): string {
    return densityIsActive(value)
      ? 'mail-list-settings-menu__item mail-list-settings-menu__item--active'
      : 'mail-list-settings-menu__item'
  }

  function updateSearchBuilderMatchMode(value: string | string[]): void {
    searchBuilderState.value = mailListSearchBuilderSetMatchMode(searchBuilderState.value, value)
  }

  function updateSearchBuilderField(value: string | string[]): void {
    searchBuilderState.value = mailListSearchBuilderSetField(searchBuilderState.value, value)
  }

  function selectSearchBuilderFieldGroup(groupId: string): void {
    activeSearchBuilderGroupId.value = groupId
  }

  function selectSearchBuilderField(groupId: string, value: string): void {
    activeSearchBuilderGroupId.value = groupId
    updateSearchBuilderField(value)
  }

  function searchBuilderFieldIsActive(value: string): boolean {
    return searchBuilderState.value.field === value
  }

  function updateSearchBuilderOperator(value: string | string[]): void {
    searchBuilderState.value = mailListSearchBuilderSetOperator(searchBuilderState.value, value)
  }

  function updateSearchBuilderValue(value: string): void {
    searchBuilderState.value = mailListSearchBuilderSetValue(searchBuilderState.value, value)
  }

  function selectSearchBuilderPreset(value: string): void {
    updateSearchBuilderValue(value)
  }

  function addSearchBuilderClause(): void {
    searchBuilderState.value = mailListSearchBuilderAddClause(searchBuilderState.value)
  }

  function removeSearchBuilderClause(clauseId: string): void {
    searchBuilderState.value = mailListSearchBuilderRemoveClause(searchBuilderState.value, clauseId)
  }

  function clearSearchBuilder(): void {
    searchBuilderState.value = mailListSearchBuilderClear()
  }

  function togglePlainSearch(): void {
    isPlainSearchOpen.value = !isPlainSearchOpen.value
  }

  function updateSearchQuery(query: string): void {
    actions.updateSearchQuery(query)
  }

  function clearSearchQuery(): void {
    actions.updateSearchQuery('')
  }

  function handleBodyScroll(event: Event): void {
    if (!props.hasMoreItems || props.isLoadingMore) return
    if (!(event.currentTarget instanceof HTMLElement)) return

    const remainingScrollPx = event.currentTarget.scrollHeight - event.currentTarget.scrollTop - event.currentTarget.clientHeight
    if (remainingScrollPx <= loadMoreScrollThresholdPx) {
      actions.loadMore()
    }
  }

  function handleSyncProgressVisibilityChange(isVisible: boolean): void {
    syncProgressVisible.value = isVisible
  }

  function applySearchBuilder(): void {
    if (!searchBuilderCanApply.value) return
    searchBuilderState.value = committedSearchBuilderState(searchBuilderState.value)
    isSearchBuilderOpen.value = false
  }

  function saveSearchBuilderFilter(): void {
    const name = savedFilterName.value.trim()
    if (!name || !searchBuilderCanApply.value) return

    const committedState = committedSearchBuilderState(searchBuilderState.value)
    const filterId = `saved-filter:${nextSavedFilterId.value}`
    nextSavedFilterId.value += 1
    const savedFilter = createSavedFilter(savedFilters.value, name, committedState, filterId)
    if (!savedFilter) return

    searchBuilderState.value = committedState
    savedFilters.value = savedFilter.filters
    activeMailViewId.value = filterId
    savedFilterName.value = ''
  }

  function selectMailView(option: TreeSelectOption): void {
    if (typeof option.value === 'string' && isMailListViewId(option.value)) {
      activeMailViewId.value = option.value
      return
    }

    if (typeof option.value !== 'string') return
    const filter = findSavedFilter(savedFilters.value, option.value)
    if (filter) {
      searchBuilderState.value = cloneSearchBuilderState(filter.state)
    }
  }

  function selectItem(item: MailListItemModel): void {
    actions.selectItem(item)
  }

  function refresh(): void {
    actions.refresh()
  }

  function loadMore(): void {
    actions.loadMore()
  }

  function importMailFile(file: File): void {
    actions.importMailFile(file)
  }

  return {
    activeDensity,
    activeMailViewId,
    activeSearchBuilderGroupId,
    isSearchBuilderOpen,
    isPlainSearchOpen,
    syncProgressVisible,
    searchBuilderState,
    savedFilterName,
    searchBuilderCanAdd,
    searchBuilderCanApply,
    searchBuilderCanSave,
    searchBuilderClauseViews,
    searchBuilderFieldGroups,
    activeSearchBuilderFieldGroup,
    searchBuilderOperatorOptions,
    searchBuilderPresetOptions,
    searchBuilderValuePlaceholder,
    searchBuilderValueSuggestions,
    visibleItems,
    builderSearchQuery,
    plainSearchQuery,
    plainSearchIsActive,
    plainSearchButtonClass,
    mailViewOptions,
    localizedSearchMatchModeItems,
    densityIsActive,
    densityMenuItemClass,
    selectDensity,
    updateSearchBuilderMatchMode,
    updateSearchBuilderField,
    selectSearchBuilderFieldGroup,
    selectSearchBuilderField,
    searchBuilderFieldIsActive,
    updateSearchBuilderOperator,
    updateSearchBuilderValue,
    selectSearchBuilderPreset,
    addSearchBuilderClause,
    removeSearchBuilderClause,
    clearSearchBuilder,
    togglePlainSearch,
    handleBodyScroll,
    handleSyncProgressVisibilityChange,
    applySearchBuilder,
    saveSearchBuilderFilter,
    selectMailView,
    updateSearchQuery,
    clearSearchQuery,
    selectItem,
    refresh,
    loadMore,
    importMailFile,
  }
}
