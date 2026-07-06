<script setup lang="ts">
import { computed, ref, useId, watch } from 'vue'
import { useI18n } from '@/platform/i18n'
import {
  Button,
  Combobox,
  DropdownMenu,
  DropdownMenuItem,
  Icon,
  NoSearchResultsState,
  Popover,
  ToggleGroup,
  TreeSelect,
  type TreeSelectOption
} from '@/shared/ui'
import '../communicationDomainElements.css'
import MailListItem from './MailListItem.vue'
import MailSyncProgress from './MailSyncProgress.vue'
import type { MailSyncStatus } from '../../types/communications'
import {
  mailListDensityToggleItems,
  type MailListItemDensity,
  type MailListItemModel
} from './mailElements'
import {
  createMailListSearchBuilderState,
  mailListSearchBuilderAddClause,
  mailListSearchBuilderCanAdd,
  mailListSearchBuilderCanApply,
  mailListSearchBuilderClauseViews,
  mailListSearchBuilderClear,
  mailListSearchBuilderQuery,
  mailListSearchBuilderRemoveClause,
  mailListSearchBuilderSetField,
  mailListSearchBuilderSetMatchMode,
  mailListSearchBuilderSetOperator,
  mailListSearchBuilderSetValue,
  mailListSearchBuilderOperatorItems,
  mailListSearchBuilderPresetItems,
  mailListSearchLocalizedToggleItems,
  mailListSearchFieldGroups,
  mailListSearchFieldItem,
  mailListSearchMatchModeItems,
  mailListItemsForSearch,
  type MailListSearchBuilderClause,
  type MailListSearchBuilderState
} from './mailSearchBuilder'
import { mailListSearchBuilderValueSuggestions } from './mailSearchSuggestions'
import {
  isMailListViewId,
  mailListItemsForView,
  mailListTreeSelectOptions,
  type MailListViewId
} from './mailListViews'

type MailListSavedFilter = {
  id: string
  name: string
  state: MailListSearchBuilderState
}

const props = defineProps<{
  items: readonly MailListItemModel[]
  hasMoreItems?: boolean
  isLoadingMore?: boolean
  searchQuery?: string
  syncStatus?: MailSyncStatus | null
}>()

const emit = defineEmits<{
  compose: []
  'load-more': []
  refresh: []
  'select-item': [item: MailListItemModel]
  'update-search-query': [query: string]
}>()

const { t } = useI18n()
const loadMoreScrollThresholdPx = 320
const plainSearchInputId = `mail-plain-search-${useId()}`
const searchBuilderValueInputId = `mail-search-builder-value-${useId()}`
const searchBuilderFilterNameInputId = `mail-search-builder-filter-name-${useId()}`
const activeDensity = ref<MailListItemDensity>('comfortable')
const activeMailViewId = ref<MailListViewId | string>('mail:all')
const activeSearchBuilderGroupId = ref(mailListSearchFieldGroups[0]?.id ?? 'text')
const isSearchBuilderOpen = ref(false)
const isPlainSearchOpen = ref(Boolean(props.searchQuery?.trim()))
const searchBuilderState = ref<MailListSearchBuilderState>(createMailListSearchBuilderState())
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
  const savedFilterOptions: TreeSelectOption[] = []

  for (const filter of savedFilters.value) {
    savedFilterOptions.push({
      value: filter.id,
      label: filter.name,
      icon: 'tabler:filter-check'
    })
  }

  if (savedFilterOptions.length === 0) {
    savedFilterOptions.push({
      value: 'saved-filters:empty',
      label: t('No saved filters yet'),
      icon: 'tabler:circle-dashed',
      disabled: true
    })
  }

  return mailListTreeSelectOptions(listItems.value, savedFilterOptions, t, Boolean(props.hasMoreItems))
})
const searchBuilderCanAdd = computed(() => mailListSearchBuilderCanAdd(searchBuilderState.value))
const searchBuilderCanApply = computed(() => mailListSearchBuilderCanApply(searchBuilderState.value))
const searchBuilderCanSave = computed(() => searchBuilderCanApply.value && savedFilterName.value.trim().length > 0)
const searchBuilderClauseViews = computed(() => mailListSearchBuilderClauseViews(searchBuilderState.value))
const searchBuilderFieldGroups = computed(() => mailListSearchFieldGroups)
const activeSearchBuilderFieldGroup = computed(() => {
  return searchBuilderFieldGroups.value.find((group) => group.id === activeSearchBuilderGroupId.value)
    ?? searchBuilderFieldGroups.value[0]
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
  () => props.searchQuery,
  (query) => {
    if (query?.trim()) isPlainSearchOpen.value = true
  }
)

function selectDensity(value: MailListItemDensity): void {
  activeDensity.value = value
}

function densityIsActive(value: MailListItemDensity): boolean {
  return activeDensity.value === value
}

function densityMenuItemClass(value: MailListItemDensity): string {
  if (densityIsActive(value)) {
    return 'mail-list-settings-menu__item mail-list-settings-menu__item--active'
  }

  return 'mail-list-settings-menu__item'
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

function updatePlainSearchQuery(event: Event): void {
  emit('update-search-query', (event.target as HTMLInputElement).value)
}

function clearPlainSearchQuery(): void {
  emit('update-search-query', '')
}

function handleBodyScroll(event: Event): void {
  if (!props.hasMoreItems || props.isLoadingMore) return

  const target = event.currentTarget
  if (!(target instanceof HTMLElement)) return

  const remainingScrollPx = target.scrollHeight - target.scrollTop - target.clientHeight
  if (remainingScrollPx <= loadMoreScrollThresholdPx) emit('load-more')
}

function committedSearchBuilderState(state: MailListSearchBuilderState): MailListSearchBuilderState {
  if (!mailListSearchBuilderCanAdd(state)) return state
  return mailListSearchBuilderAddClause(state)
}

function cloneSearchBuilderState(state: MailListSearchBuilderState): MailListSearchBuilderState {
  const clauses: MailListSearchBuilderClause[] = []
  for (const clause of state.clauses) {
    clauses.push({ ...clause })
  }
  return { ...state, clauses }
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
  searchBuilderState.value = committedState
  savedFilters.value = [
    ...savedFilters.value,
    {
      id: filterId,
      name,
      state: cloneSearchBuilderState(committedState)
    }
  ]
  activeMailViewId.value = filterId
  savedFilterName.value = ''
}

function selectMailView(option: TreeSelectOption): void {
  if (typeof option.value === 'string' && isMailListViewId(option.value)) {
    activeMailViewId.value = option.value
    return
  }

  for (const filter of savedFilters.value) {
    if (filter.id === option.value) {
      searchBuilderState.value = cloneSearchBuilderState(filter.state)
      return
    }
  }
}
</script>

<template>
	<div class="mail-list-stack">
		<section class="mail-list-action-card" :aria-label="t('Mail actions')">
			<Button class="mail-list-action-card__compose" icon="tabler:edit" size="sm" @click="emit('compose')">
				{{ t('Compose') }}
			</Button>
			<div class="mail-list-action-card__tools">
				<Button
					class="mail-list-action-card__tool hermes-icon-button"
					variant="outline"
					size="sm"
					icon="tabler:refresh"
					:aria-label="t('Refresh')"
					:title="t('Refresh')"
					@click="emit('refresh')"
				/>
				<DropdownMenu
					align="end"
					:side-offset="8"
					class="mail-list-settings-menu"
				>
					<template #trigger>
						<Button
							class="mail-list-action-card__tool hermes-icon-button"
							variant="outline"
							size="sm"
							icon="tabler:settings"
							:aria-label="t('Settings')"
							:title="t('Settings')"
						/>
					</template>
					<div class="mail-list-settings-menu__body" :aria-label="t('Mail list settings')">
						<span class="mail-list-settings-menu__label">{{ t('List display') }}</span>
						<DropdownMenuItem
							v-for="density in mailListDensityToggleItems"
							:key="density.value"
							:icon="density.icon"
							:class="densityMenuItemClass(density.value)"
							@click="selectDensity(density.value)"
						>
							<span class="mail-list-settings-menu__item-label">{{ t(density.label) }}</span>
							<Icon
								v-if="densityIsActive(density.value)"
								icon="tabler:check"
								size="0.95rem"
								class="mail-list-settings-menu__check"
							/>
						</DropdownMenuItem>
					</div>
				</DropdownMenu>
			</div>
		</section>
		<section class="communication-workspace-panel communication-workspace-panel--inbox" :aria-label="t('Mail list')">
			<header class="communication-workspace-panel__header">
				<div class="mail-list-view-select-row">
					<TreeSelect
						v-model="activeMailViewId"
						class="mail-list-view-select"
						:options="mailViewOptions"
						:placeholder="t('Select mail view')"
						:aria-label="t('Mail view')"
						:empty-label="t('No options')"
						@select="selectMailView"
					/>
					<Button
						:class="plainSearchButtonClass"
						variant="outline"
						icon="tabler:search"
						:aria-expanded="isPlainSearchOpen"
						:aria-label="t('Search mail')"
						:title="t('Search mail')"
						@click="togglePlainSearch"
					/>
					<Popover
						v-model:open="isSearchBuilderOpen"
						align="start"
						:side-offset="8"
						class="mail-search-builder-popover"
						:close-label="t('Close search builder')"
					>
						<template #trigger>
							<Button
								class="mail-list-view-select-row__builder hermes-icon-button"
								variant="outline"
								icon="tabler:adjustments-search"
								:aria-label="t('Search builder')"
								:title="t('Search builder')"
							/>
						</template>
						<div class="mail-search-builder" :aria-label="t('Mail search builder')">
							<div class="mail-search-builder__header">
								<Icon icon="tabler:filter-search" size="1.1rem" class="mail-search-builder__header-icon" />
								<strong class="mail-search-builder__title">{{ t('Search builder') }}</strong>
							</div>
							<div class="mail-search-builder__match-row">
								<span class="mail-search-builder__label">{{ t('Match') }}</span>
								<ToggleGroup
									:model-value="searchBuilderState.matchMode"
									:items="localizedSearchMatchModeItems"
									class="mail-search-builder__toggle hermes-toggle-group--tabs"
									:aria-label="t('Search match mode')"
									@update:model-value="updateSearchBuilderMatchMode"
								/>
							</div>
							<div class="mail-search-builder__field-layout">
								<div class="mail-search-builder__section">
									<span class="mail-search-builder__label">{{ t('Field') }}</span>
									<div class="mail-search-builder__group-tabs" role="tablist" :aria-label="t('Search field group')">
										<button
											v-for="group in searchBuilderFieldGroups"
											:key="group.id"
											type="button"
											role="tab"
											:class="[
												'mail-search-builder__group-tab',
												activeSearchBuilderGroupId === group.id && 'mail-search-builder__group-tab--active'
											]"
											:aria-selected="activeSearchBuilderGroupId === group.id"
											@click="selectSearchBuilderFieldGroup(group.id)"
										>
											{{ t(group.label) }}
										</button>
									</div>
									<div class="mail-search-builder__field-options">
										<button
											v-for="field in activeSearchBuilderFieldGroup?.fields ?? []"
											:key="field.value"
											type="button"
											:class="[
												'mail-search-builder__field-option',
												searchBuilderFieldIsActive(field.value) && 'mail-search-builder__field-option--active'
											]"
											:aria-pressed="searchBuilderFieldIsActive(field.value)"
											:title="t(field.placeholder)"
											@click="selectSearchBuilderField(activeSearchBuilderFieldGroup?.id ?? 'text', field.value)"
										>
											{{ t(field.label) }}
										</button>
									</div>
								</div>
								<div class="mail-search-builder__rule-panel">
									<div class="mail-search-builder__control">
										<span class="mail-search-builder__label">{{ t('Operator') }}</span>
										<ToggleGroup
											:model-value="searchBuilderState.operator"
											:items="searchBuilderOperatorOptions"
											class="mail-search-builder__toggle hermes-toggle-group--tabs"
											:aria-label="t('Search operator')"
											@update:model-value="updateSearchBuilderOperator"
										/>
									</div>
									<div
										v-if="searchBuilderPresetOptions.length"
										class="mail-search-builder__presets"
										:aria-label="t('Search value presets')"
									>
										<button
											v-for="preset in searchBuilderPresetOptions"
											:key="preset.value"
											type="button"
											class="mail-search-builder__preset"
											@click="selectSearchBuilderPreset(preset.value)"
										>
											{{ preset.label }}
										</button>
									</div>
								</div>
							</div>
							<div class="mail-search-builder__value-row">
								<Combobox
									:id="searchBuilderValueInputId"
									:model-value="searchBuilderState.value"
									class="mail-search-builder__value"
									:options="searchBuilderValueSuggestions"
									:placeholder="searchBuilderValuePlaceholder"
									:aria-label="t('Builder search value')"
									@update:model-value="updateSearchBuilderValue"
								/>
								<Button
									class="mail-search-builder__add"
									variant="secondary"
									icon="tabler:plus"
									:disabled="!searchBuilderCanAdd"
									@click="addSearchBuilderClause"
								>
									{{ t('Add') }}
								</Button>
							</div>
							<div class="mail-search-builder__clauses" :aria-label="t('Search clauses')">
								<div
									v-for="clause in searchBuilderClauseViews"
									:key="clause.id"
									:class="['mail-search-builder__clause', clause.pending && 'mail-search-builder__clause--pending']"
								>
									<span
										v-for="token in clause.tokens"
										:key="token.id"
										class="mail-search-builder__token"
									>
										{{ t(token.value) }}
									</span>
									<button
										v-if="!clause.pending"
										class="mail-search-builder__remove"
										type="button"
										:aria-label="t('Remove search clause')"
										@click="removeSearchBuilderClause(clause.id)"
									>
										<Icon icon="tabler:x" size="0.85rem" />
									</button>
								</div>
								<span v-if="!searchBuilderClauseViews.length" class="mail-search-builder__empty">
									{{ t('No clauses yet') }}
								</span>
							</div>
							<div class="mail-search-builder__save-row">
								<label class="mail-search-builder__filter-name">
									<span class="mail-search-builder__label">{{ t('Filter name') }}</span>
									<input
										:id="searchBuilderFilterNameInputId"
										v-model="savedFilterName"
										class="mail-search-builder__name-input"
										type="text"
										:placeholder="t('Filter name')"
										:aria-label="t('Search filter name')"
									/>
								</label>
								<Button
									class="mail-search-builder__save"
									variant="secondary"
									icon="tabler:device-floppy"
									:disabled="!searchBuilderCanSave"
									@click="saveSearchBuilderFilter"
								>
									{{ t('Save filter') }}
								</Button>
							</div>
							<div class="mail-search-builder__actions">
								<Button variant="ghost" @click="clearSearchBuilder">{{ t('Clear') }}</Button>
								<Button
									icon="tabler:check"
									:disabled="!searchBuilderCanApply"
									@click="applySearchBuilder"
								>
									{{ t('Apply') }}
								</Button>
							</div>
						</div>
					</Popover>
				</div>
				<div v-if="isPlainSearchOpen" class="mail-list-plain-search" role="search">
					<Icon icon="tabler:search" size="1rem" class="mail-list-plain-search__icon" />
					<input
						:id="plainSearchInputId"
						:value="plainSearchQuery"
						class="mail-list-plain-search__input"
						type="search"
						:placeholder="t('Address, subject or body')"
						:aria-label="t('Search address, subject or body')"
						@input="updatePlainSearchQuery"
					/>
					<button
						v-if="plainSearchIsActive"
						type="button"
						class="mail-list-plain-search__clear"
						:aria-label="t('Clear search')"
						:title="t('Clear search')"
						@click="clearPlainSearchQuery"
					>
						<Icon icon="tabler:x" size="0.9rem" />
					</button>
				</div>
			</header>
			<div class="communication-workspace-panel__body" @scroll="handleBodyScroll">
				<div v-if="visibleItems.length" class="communication-inbox-list">
					<MailListItem
						v-for="item in visibleItems"
						:key="item.id"
						:item="item"
						:density="activeDensity"
						@select="emit('select-item', $event)"
					/>
				</div>
				<NoSearchResultsState
					v-else
					class="mail-list-empty"
					:title="t('No matching mail')"
					:description="t('Try text, mail attributes or Hermes entities.')"
					:query="plainSearchQuery || builderSearchQuery"
				/>
				<footer
					v-if="hasMoreItems || isLoadingMore"
					class="mail-list-load-more"
					aria-live="polite"
				>
					<Button
						v-if="hasMoreItems && !isLoadingMore"
						variant="ghost"
						size="sm"
						icon="tabler:chevrons-down"
						@click="emit('load-more')"
					>
						{{ t('Load more') }}
					</Button>
					<span v-else class="mail-list-load-more__status">
						<Icon icon="tabler:loader-2" size="0.95rem" />
						{{ t('Loading more') }}
					</span>
				</footer>
			</div>
		</section>
		<MailSyncProgress :status="syncStatus" />
	</div>
</template>
