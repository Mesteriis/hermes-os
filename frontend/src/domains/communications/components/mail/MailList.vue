<script setup lang="ts">
import { computed, ref, useId } from 'vue'
import { useI18n } from '@/platform/i18n'
import { Badge, Button, Combobox, Icon, NoSearchResultsState, Popover, ToggleGroup } from '@/shared/ui'
import '../communicationDomainElements.css'
import MailListItem from './MailListItem.vue'
import {
  mailListAccountOptions,
  mailListAllAccountsOptionId,
  mailListDensityToggleItems,
  mailListItemsForAccount,
  mailListItemDensityOptions,
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
  mailListSearchBuilderCommittedClauseViews,
  mailListSearchBuilderDraftTokens,
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
  mailListSearchPlaceholder,
  type MailListSearchBuilderState
} from './mailSearchBuilder'
import { mailListSearchBuilderValueSuggestions } from './mailSearchSuggestions'

const props = defineProps<{
  items: readonly MailListItemModel[]
}>()

const { t } = useI18n()
const searchBuilderValueInputId = `mail-search-builder-value-${useId()}`
const activeDensity = ref<MailListItemDensity>('comfortable')
const activeAccountId = ref(mailListAllAccountsOptionId)
const activeSearchBuilderGroupId = ref(mailListSearchFieldGroups[0]?.id ?? 'text')
const isSearchBuilderOpen = ref(false)
const searchBuilderState = ref<MailListSearchBuilderState>(createMailListSearchBuilderState())

const searchQuery = computed(() => mailListSearchBuilderQuery(searchBuilderState.value))
const accountOptions = computed(() => mailListAccountOptions(props.items))
const accountItems = computed(() => mailListItemsForAccount(props.items, activeAccountId.value))
const visibleItems = computed(() => mailListItemsForSearch(accountItems.value, searchQuery.value))
const searchBuilderCanAdd = computed(() => mailListSearchBuilderCanAdd(searchBuilderState.value))
const searchBuilderCanApply = computed(() => mailListSearchBuilderCanApply(searchBuilderState.value))
const searchBuilderClauseViews = computed(() => mailListSearchBuilderClauseViews(searchBuilderState.value))
const searchBuilderCommittedClauseViews = computed(() => mailListSearchBuilderCommittedClauseViews(searchBuilderState.value))
const searchBuilderDraftTokens = computed(() => mailListSearchBuilderDraftTokens(searchBuilderState.value))
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
  return mailListSearchBuilderValueSuggestions(accountItems.value, searchBuilderState.value)
})
const badgeLabel = computed(() => {
  if (!searchQuery.value.trim()) return t('{count} threads', { count: accountItems.value.length })
  return t('{visible} of {total}', { visible: visibleItems.value.length, total: accountItems.value.length })
})

function updateDensity(value: string | string[]): void {
  if (typeof value !== 'string') return
  if (!mailListItemDensityOptions.includes(value as MailListItemDensity)) return

  activeDensity.value = value as MailListItemDensity
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

function handleSearchBuilderInput(event: Event): void {
  const target = event.target as HTMLInputElement
  updateSearchBuilderValue(target.value)
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

function applySearchBuilder(): void {
  if (!searchBuilderCanApply.value) return
  isSearchBuilderOpen.value = false
}
</script>

<template>
	<section class="communication-workspace-panel communication-workspace-panel--inbox" :aria-label="t('Mail list')">
		<header class="communication-workspace-panel__header">
			<div class="communication-workspace-panel__title-row">
				<label class="mail-list-account-select">
					<span class="mail-list-account-select__icon" aria-hidden="true">
						<Icon icon="tabler:mail" size="1rem" />
					</span>
					<span class="mail-list-token-search__sr">{{ t('Mail account') }}</span>
					<select
						v-model="activeAccountId"
						class="mail-list-account-select__control"
						:aria-label="t('Mail account')"
					>
						<option
							v-for="option in accountOptions"
							:key="option.id"
							:value="option.id"
						>
							{{ option.id === mailListAllAccountsOptionId ? t(option.label) : option.label }}
						</option>
					</select>
					<Icon icon="tabler:chevron-down" size="0.95rem" class="mail-list-account-select__chevron" />
				</label>
				<Badge variant="accent">{{ badgeLabel }}</Badge>
			</div>
			<div class="mail-list-toolbar" :aria-label="t('Mail list controls')">
				<div class="mail-list-token-search" role="search" :aria-label="t('Mail structured search')">
					<Icon icon="tabler:search" size="1.15rem" class="mail-list-token-search__icon" />
					<div
						v-if="searchBuilderCommittedClauseViews.length"
						class="mail-list-token-search__clauses"
						:aria-label="t('Applied search clauses')"
					>
						<span
							v-for="clause in searchBuilderCommittedClauseViews"
							:key="clause.id"
							class="mail-list-token-search__clause"
						>
							<span
								v-for="token in clause.tokens"
								:key="token.id"
								class="mail-list-token-search__token"
							>
								{{ t(token.value) }}
							</span>
						</span>
					</div>
					<div class="mail-list-token-search__draft">
						<span
							v-for="token in searchBuilderDraftTokens"
							:key="token.id"
							class="mail-list-token-search__token"
						>
							{{ t(token.value) }}
						</span>
						<label class="mail-list-token-search__value">
							<span class="mail-list-token-search__sr">{{ t('Search value') }}</span>
							<input
								:value="searchBuilderState.value"
								class="mail-list-token-search__input"
								type="search"
								:placeholder="t(mailListSearchPlaceholder)"
								:aria-label="t('Mail search value')"
								@input="handleSearchBuilderInput"
							/>
						</label>
					</div>
					<button
						v-if="searchQuery"
						class="mail-list-token-search__clear"
						type="button"
						:aria-label="t('Clear mail search')"
						@click="clearSearchBuilder"
					>
						<Icon icon="tabler:x" size="1rem" />
					</button>
				</div>
				<Popover
					v-model:open="isSearchBuilderOpen"
					align="start"
					:side-offset="8"
					class="mail-search-builder-popover"
					:close-label="t('Close search builder')"
				>
					<template #trigger>
						<Button
							class="mail-list-toolbar__builder hermes-icon-button"
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
				<ToggleGroup
					:model-value="activeDensity"
					:items="mailListDensityToggleItems"
					class="mail-list-toolbar__density hermes-toggle-group--tabs"
					:aria-label="t('Mail list density')"
					@update:model-value="updateDensity"
				/>
			</div>
		</header>
		<div class="communication-workspace-panel__body">
			<div v-if="visibleItems.length" class="communication-inbox-list">
				<MailListItem
					v-for="item in visibleItems"
					:key="item.id"
					:item="item"
					:density="activeDensity"
				/>
			</div>
			<NoSearchResultsState
				v-else
				class="mail-list-empty"
				:title="t('No matching mail')"
				:description="t('Try text, mail attributes or Hermes entities.')"
				:query="searchQuery"
			/>
		</div>
	</section>
</template>
