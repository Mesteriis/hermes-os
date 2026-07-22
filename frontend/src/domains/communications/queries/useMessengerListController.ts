import { computed, ref } from 'vue'
import { useI18n } from '@/platform/i18n'
import {
  messengerItemsForSearch,
  messengerItemsForView,
  messengerListDensityOptions,
  messengerListViewOptions,
  messengerProviderViewId,
  type MessengerListItemDensity,
  type MessengerListItemModel,
} from '../components/messengers/messengerElements'

export interface MessengerListControllerProps {
  errorMessage?: string
  isLoading?: boolean
  isRefreshing?: boolean
  items: readonly MessengerListItemModel[]
  selectedId?: string
}

export interface MessengerListControllerActions {
  refresh: () => void
  selectItem: (item: MessengerListItemModel) => void
}

export function useMessengerListController(
  props: Readonly<MessengerListControllerProps>,
  actions: MessengerListControllerActions,
) {
  const { t } = useI18n()

  const activeDensity = ref<MessengerListItemDensity>('comfortable')
  const activeViewId = ref(messengerProviderViewId('all'))
  const searchValue = ref('')

  const viewOptions = computed(() => messengerListViewOptions(props.items, t))
  const visibleItems = computed(() => messengerItemsForSearch(
    messengerItemsForView(props.items, activeViewId.value),
    searchValue.value,
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

  function handleRefresh(): void {
    actions.refresh()
  }

  function handleSelect(item: MessengerListItemModel): void {
    actions.selectItem(item)
  }

  function clearSearch(): void {
    searchValue.value = ''
  }

  return {
    t,
    activeDensity,
    activeViewId,
    searchValue,
    viewOptions,
    visibleItems,
    messengerListDensityOptions,
    messengerProviderViewId,
    densityMenuItemClass,
    selectDensity,
    densityIsActive,
    handleRefresh,
    handleSelect,
    clearSearch,
  }
}
