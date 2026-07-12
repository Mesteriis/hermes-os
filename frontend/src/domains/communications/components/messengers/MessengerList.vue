<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '@/platform/i18n'
import { Button, DropdownMenu, DropdownMenuItem, Icon, NoSearchResultsState, TreeSelect } from '@/shared/ui'
import '../communicationDomainElements.css'
import MessengerListItem from './MessengerListItem.vue'
import {
  messengerItemsForSearch,
  messengerItemsForView,
  messengerListDensityOptions,
  messengerListViewOptions,
  messengerProviderViewId,
  type MessengerListItemDensity,
  type MessengerListItemModel
} from './messengerElements'

const props = defineProps<{
  errorMessage?: string
  isLoading?: boolean
  isRefreshing?: boolean
  items: readonly MessengerListItemModel[]
  selectedId?: string
}>()

const emit = defineEmits<{
  refresh: []
  select: [item: MessengerListItemModel]
}>()

const { t } = useI18n()
const activeDensity = ref<MessengerListItemDensity>('comfortable')
const activeViewId = ref(messengerProviderViewId('all'))
const searchValue = ref('')

const viewOptions = computed(() => messengerListViewOptions(props.items, t))
const visibleItems = computed(() => messengerItemsForSearch(
	messengerItemsForView(props.items, activeViewId.value),
	searchValue.value
))

function selectDensity(value: MessengerListItemDensity): void {
  activeDensity.value = value
}

function densityIsActive(value: MessengerListItemDensity): boolean {
  return activeDensity.value === value
}

function densityMenuItemClass(value: MessengerListItemDensity): string {
  if (densityIsActive(value)) {
    return 'mail-list-settings-menu__item mail-list-settings-menu__item--active'
  }

  return 'mail-list-settings-menu__item'
}
</script>

<template>
	<div class="messenger-list-stack">
		<section class="messenger-list-action-card" :aria-label="t('Messenger actions')">
			<div class="messenger-list-search" role="search">
				<Icon icon="tabler:search" size="1rem" class="messenger-list-search__icon" />
				<span class="messenger-list-search__label">{{ t('Search messengers') }}</span>
				<input
					v-model="searchValue"
					class="messenger-list-search__input"
					type="search"
					:placeholder="t('Search Telegram, WhatsApp, dialogs')"
					:aria-label="t('Search messengers')"
				/>
				<button
					v-if="searchValue"
					class="messenger-list-search__clear"
					type="button"
					:aria-label="t('Clear search')"
					:title="t('Clear search')"
					@click="searchValue = ''"
				>
					<Icon icon="tabler:x" size="1rem" />
				</button>
			</div>
			<div class="messenger-list-action-card__tools">
				<Button
					class="messenger-list-action-card__tool hermes-icon-button"
					variant="outline"
					size="sm"
					icon="tabler:refresh"
					:aria-label="t('Refresh')"
					:disabled="isRefreshing"
					:title="t('Refresh')"
					@click="emit('refresh')"
				/>
				<DropdownMenu align="end" :side-offset="8" class="mail-list-settings-menu">
					<template #trigger>
						<Button
							class="messenger-list-action-card__tool hermes-icon-button"
							variant="outline"
							size="sm"
							icon="tabler:settings"
							:aria-label="t('Settings')"
							:title="t('Settings')"
						/>
					</template>
					<div class="mail-list-settings-menu__body" :aria-label="t('Messenger list settings')">
						<span class="mail-list-settings-menu__label">{{ t('List display') }}</span>
						<DropdownMenuItem
							v-for="density in messengerListDensityOptions"
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

		<section class="communication-workspace-panel communication-workspace-panel--inbox" :aria-label="t('Messenger list')">
			<header class="communication-workspace-panel__header">
				<TreeSelect
					v-model="activeViewId"
					class="messenger-list-view-select"
					:options="viewOptions"
					:placeholder="t('Select messenger view')"
					:aria-label="t('Messenger view')"
					:empty-label="t('No options')"
				/>
			</header>
			<div class="communication-workspace-panel__body">
				<div v-if="visibleItems.length" class="messenger-list">
					<MessengerListItem
						v-for="item in visibleItems"
						:key="item.id"
						:item="item"
						:density="activeDensity"
						:selected="selectedId ? item.id === selectedId : item.selected"
						@select="emit('select', $event)"
					/>
				</div>
				<p v-else-if="isLoading" class="messenger-list-empty" role="status">
					{{ t('Loading dialogs') }}
				</p>
				<div v-else-if="errorMessage" class="messenger-list-empty" role="alert">
					<p>{{ t('Could not load dialogs') }}</p>
					<Button size="sm" variant="outline" @click="emit('refresh')">{{ t('Retry') }}</Button>
				</div>
				<NoSearchResultsState
					v-else
					class="messenger-list-empty"
					:title="t('No matching dialogs')"
					:description="t('Try another messenger view or saved filter.')"
				/>
			</div>
		</section>
	</div>
</template>
