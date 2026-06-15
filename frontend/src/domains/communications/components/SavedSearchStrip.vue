<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useForm } from 'vee-validate'
import { useVirtualizer } from '@tanstack/vue-virtual'
import Dialog from '../../../shared/ui/Dialog.vue'
import Icon from '../../../shared/ui/Icon.vue'
import SavedSearchRuleGroupEditor from './SavedSearchRuleGroupEditor.vue'
import {
  useCreateSavedSearchMutation,
  useDeleteSavedSearchMutation,
  useSavedSearchesQuery,
  useUpdateSavedSearchMutation
} from '../queries/useCommunicationsQuery'
import {
  composeSavedSearchRuleTreeQuery,
  createSavedSearchRuleGroup,
  flattenSavedSearchRuleTree,
  parseSavedSearchQuery,
  savedSearchDeleteDialogCopy,
  savedSearchChannelOptions,
  savedSearchFilterChips,
  savedSearchFormDefaults,
  normalizeSavedSearchBuilderState,
  savedSearchFormToInput,
  savedSearchLocalStateOptions,
  savedSearchMessageCountLabel,
  savedSearchPresetOptions,
  validateSavedSearchRuleTree,
  savedSearchWorkflowOptions,
  savedSearchVeeValidationSchema,
  type SavedSearchRuleGroup,
  type SavedSearchPresetOption,
  type SavedSearchFormValues
} from '../forms/savedSearchForm'
import { useSavedSearchMailListPrefetch } from '../queries/mailPrefetch'
import type { MailSavedSearch } from '../types/savedSearches'
import type { LocalMessageState, WorkflowState } from '../types/communications'
import './SavedSearchStrip.css'

const props = defineProps<{
  accountId: string | null
  activeId: string
  currentQuery: string
  currentWorkflowState: WorkflowState | ''
  currentLocalState: LocalMessageState
  currentChannelKind: string
}>()

const emit = defineEmits<{
  select: [savedSearch: MailSavedSearch]
  deleted: [savedSearch: MailSavedSearch]
}>()

const {
  data: smartFolderData,
  fetchNextPage: fetchNextSmartFolderPage,
  hasNextPage: hasNextSmartFolderPage,
  isFetchingNextPage: isFetchingNextSmartFolderPage,
  isLoading: isSmartFolderLoading
} = useSavedSearchesQuery(() => true, () => props.accountId || undefined)
const {
  data: savedSearchData,
  fetchNextPage: fetchNextSavedSearchPage,
  hasNextPage: hasNextSavedSearchPage,
  isFetchingNextPage: isFetchingNextSavedSearchPage,
  isLoading: isSavedSearchLoading
} = useSavedSearchesQuery(() => false, () => props.accountId || undefined)

const smartFolders = computed(() => smartFolderData.value ?? [])
const savedSearches = computed(() => savedSearchData.value ?? [])
const isLoading = computed(() => isSmartFolderLoading.value || isSavedSearchLoading.value)
const dialogOpen = ref(false)
const editingSearch = ref<MailSavedSearch | null>(null)
const deleteDialogOpen = ref(false)
const deletingSearch = ref<MailSavedSearch | null>(null)
const searchRuleTree = ref<SavedSearchRuleGroup>(createSavedSearchRuleGroup('all'))
const deleteError = ref('')
const createMutation = useCreateSavedSearchMutation()
const updateMutation = useUpdateSavedSearchMutation()
const deleteMutation = useDeleteSavedSearchMutation()
const prefetchSavedSearchMailList = useSavedSearchMailListPrefetch(() => props.accountId)
const smartFolderVirtualScrollRef = ref<HTMLDivElement | null>(null)
const savedSearchVirtualScrollRef = ref<HTMLDivElement | null>(null)
const {
  errors,
  handleSubmit,
  resetForm,
  setFieldValue,
  values: formValues
} = useForm<SavedSearchFormValues>({
  validationSchema: savedSearchVeeValidationSchema,
  initialValues: savedSearchFormDefaults(null, false)
})
const isSaving = computed(() => createMutation.isPending.value || updateMutation.isPending.value)
const isDeleting = computed(() => deleteMutation.isPending.value)
const dialogTitle = computed(() => {
  if (editingSearch.value) return editingSearch.value.is_smart_folder ? 'Edit smart folder' : 'Edit saved search'
  return formValues.is_smart_folder ? 'New smart folder' : 'New saved search'
})
const deleteCopy = computed(() => {
  return deletingSearch.value ? savedSearchDeleteDialogCopy(deletingSearch.value) : null
})
const activeFilterChips = computed(() =>
  savedSearchFilterChips(
    {
      ...formValues,
      match_mode: searchRuleTree.value.matchMode
    },
    flattenSavedSearchRuleTree(searchRuleTree.value)
  )
)
const effectiveQueryPreview = computed(() =>
  composeSavedSearchRuleTreeQuery(formValues.query, searchRuleTree.value)
)
const ruleValidation = computed(() => validateSavedSearchRuleTree(searchRuleTree.value))
const smartFolderVirtualOptions = computed(() => ({
  count: smartFolders.value.length,
  getScrollElement: () => smartFolderVirtualScrollRef.value,
  estimateSize: () => 192,
  horizontal: true,
  overscan: 6
}))
const savedSearchVirtualOptions = computed(() => ({
  count: savedSearches.value.length,
  getScrollElement: () => savedSearchVirtualScrollRef.value,
  estimateSize: () => 192,
  horizontal: true,
  overscan: 6
}))
const smartFolderVirtualizer = useVirtualizer(smartFolderVirtualOptions)
const savedSearchVirtualizer = useVirtualizer(savedSearchVirtualOptions)
const virtualSmartFolders = computed(() => smartFolderVirtualizer.value.getVirtualItems())
const virtualSavedSearches = computed(() => savedSearchVirtualizer.value.getVirtualItems())
const smartFolderVirtualTotalSize = computed(() => smartFolderVirtualizer.value.getTotalSize())
const savedSearchVirtualTotalSize = computed(() => savedSearchVirtualizer.value.getTotalSize())

watch(dialogOpen, (open) => {
  if (!open) editingSearch.value = null
})
watch(deleteDialogOpen, (open) => {
  if (!open) deletingSearch.value = null
})

function openCreateDialog(isSmartFolder: boolean) {
  editingSearch.value = null
  const defaults = currentSearchDefaults(isSmartFolder)
  resetForm({ values: defaults })
  searchRuleTree.value = createSavedSearchRuleGroup('all')
  syncRuleTreeFromQuery(defaults.query)
  dialogOpen.value = true
}

function openEditDialog(savedSearch: MailSavedSearch) {
  editingSearch.value = savedSearch
  resetForm({ values: savedSearchFormDefaults(savedSearch) })
  syncRuleTreeFromQuery(savedSearch.query)
  dialogOpen.value = true
}

function openDeleteDialog(savedSearch: MailSavedSearch) {
  deletingSearch.value = savedSearch
  deleteError.value = ''
  deleteDialogOpen.value = true
}

function currentSearchDefaults(isSmartFolder: boolean): SavedSearchFormValues {
  return {
    ...savedSearchFormDefaults(null, isSmartFolder),
    query: props.currentQuery.trim(),
    workflow_state: props.currentWorkflowState || null,
    local_state: props.currentLocalState,
    channel_kind: props.currentChannelKind.trim()
  }
}

function handleSavedSearchPrefetch(savedSearch: MailSavedSearch) {
  void prefetchSavedSearchMailList(savedSearch)
}

function applyPreset(preset: SavedSearchPresetOption) {
  resetForm({
    values: {
      ...formValues,
      ...preset.values,
      match_mode: 'all'
    }
  })
  searchRuleTree.value = createSavedSearchRuleGroup('all')
  syncRuleTreeFromQuery(preset.values.query ?? '')
}

function syncRuleTreeFromQuery(rawQuery: string) {
  const parsed = parseSavedSearchQuery(rawQuery)
  searchRuleTree.value = parsed.tree
  updateFormQuery(parsed.plainQuery)
}

function updateFormQuery(query: string) {
  setFieldValue('query', query)
}

function normalizeQueryIntoBuilder(rawQuery: string) {
  const normalized = normalizeSavedSearchBuilderState(
    rawQuery,
    flattenSavedSearchRuleTree(searchRuleTree.value),
    searchRuleTree.value.matchMode
  )
  searchRuleTree.value = normalized.tree
  updateFormQuery(normalized.plainQuery)
}

function handleSmartFolderVirtualScroll() {
  const scrollEl = smartFolderVirtualScrollRef.value
  if (!scrollEl || !hasNextSmartFolderPage.value || isFetchingNextSmartFolderPage.value) return
  if (scrollEl.scrollLeft + scrollEl.clientWidth >= scrollEl.scrollWidth - 320) {
    void fetchNextSmartFolderPage()
  }
}

function handleSavedSearchVirtualScroll() {
  const scrollEl = savedSearchVirtualScrollRef.value
  if (!scrollEl || !hasNextSavedSearchPage.value || isFetchingNextSavedSearchPage.value) return
  if (scrollEl.scrollLeft + scrollEl.clientWidth >= scrollEl.scrollWidth - 320) {
    void fetchNextSavedSearchPage()
  }
}

const submitSavedSearch = handleSubmit(async (values) => {
  normalizeQueryIntoBuilder(values.query)
  const validation = validateSavedSearchRuleTree(searchRuleTree.value)
  if (!validation.isValid) return
  const request = savedSearchFormToInput(
    {
      ...values,
      query: composeSavedSearchRuleTreeQuery(values.query, searchRuleTree.value)
    },
    props.accountId
  )
  if (editingSearch.value) {
    await updateMutation.mutateAsync({
      savedSearchId: editingSearch.value.saved_search_id,
      request
    })
  } else {
    await createMutation.mutateAsync(request)
  }
  dialogOpen.value = false
})

async function confirmDeleteSavedSearch() {
  const savedSearch = deletingSearch.value
  if (!savedSearch) return

  deleteError.value = ''
  try {
    await deleteMutation.mutateAsync(savedSearch.saved_search_id)
    if (savedSearch.saved_search_id === props.activeId) emit('deleted', savedSearch)
    deleteDialogOpen.value = false
  } catch (e) {
    deleteError.value = e instanceof Error ? e.message : 'Saved search deletion failed'
  }
}
</script>

<template>
  <div class="saved-search-strip">
    <div v-if="isLoading" class="saved-search-skeleton" />
    <template v-else>
      <div v-if="smartFolders.length" class="saved-search-group">
        <span class="saved-search-label">Smart</span>
        <div
          ref="smartFolderVirtualScrollRef"
          class="saved-search-virtual-scroll"
          @scroll="handleSmartFolderVirtualScroll"
        >
          <div class="saved-search-virtual-track" :style="{ width: `${smartFolderVirtualTotalSize}px` }">
            <div
              v-for="virtualItem in virtualSmartFolders"
              :key="String(virtualItem.key)"
              class="saved-search-virtual-row"
              :style="{
                width: `${virtualItem.size}px`,
                transform: `translateX(${virtualItem.start}px)`
              }"
            >
              <div class="saved-search-item">
                <button
                  class="saved-search-chip"
                  :class="{ active: smartFolders[virtualItem.index].saved_search_id === activeId }"
                  type="button"
                  @mouseenter="handleSavedSearchPrefetch(smartFolders[virtualItem.index])"
                  @focus="handleSavedSearchPrefetch(smartFolders[virtualItem.index])"
                  @click="emit('select', smartFolders[virtualItem.index])"
                >
                  <Icon icon="tabler:folder-bolt" class="saved-search-icon" />
                  <span class="saved-search-name">{{ smartFolders[virtualItem.index].name }}</span>
                  <span class="saved-search-count">{{ savedSearchMessageCountLabel(smartFolders[virtualItem.index]) }}</span>
                </button>
                <button class="saved-search-tool" type="button" :title="`Edit ${smartFolders[virtualItem.index].name}`" @click="openEditDialog(smartFolders[virtualItem.index])">
                  <Icon icon="tabler:pencil" class="saved-search-icon" />
                </button>
                <button class="saved-search-tool danger" type="button" :title="`Delete ${smartFolders[virtualItem.index].name}`" @click="openDeleteDialog(smartFolders[virtualItem.index])">
                  <Icon icon="tabler:trash" class="saved-search-icon" />
                </button>
              </div>
            </div>
          </div>
        </div>
        <span v-if="isFetchingNextSmartFolderPage" class="saved-search-loading-more">Loading smart folders...</span>
      </div>
      <div v-if="savedSearches.length" class="saved-search-group">
        <span class="saved-search-label">Saved</span>
        <div
          ref="savedSearchVirtualScrollRef"
          class="saved-search-virtual-scroll"
          @scroll="handleSavedSearchVirtualScroll"
        >
          <div class="saved-search-virtual-track" :style="{ width: `${savedSearchVirtualTotalSize}px` }">
            <div
              v-for="virtualItem in virtualSavedSearches"
              :key="String(virtualItem.key)"
              class="saved-search-virtual-row"
              :style="{
                width: `${virtualItem.size}px`,
                transform: `translateX(${virtualItem.start}px)`
              }"
            >
              <div class="saved-search-item">
                <button
                  class="saved-search-chip"
                  :class="{ active: savedSearches[virtualItem.index].saved_search_id === activeId }"
                  type="button"
                  @mouseenter="handleSavedSearchPrefetch(savedSearches[virtualItem.index])"
                  @focus="handleSavedSearchPrefetch(savedSearches[virtualItem.index])"
                  @click="emit('select', savedSearches[virtualItem.index])"
                >
                  <Icon icon="tabler:search" class="saved-search-icon" />
                  <span class="saved-search-name">{{ savedSearches[virtualItem.index].name }}</span>
                  <span class="saved-search-count">{{ savedSearchMessageCountLabel(savedSearches[virtualItem.index]) }}</span>
                </button>
                <button class="saved-search-tool" type="button" :title="`Edit ${savedSearches[virtualItem.index].name}`" @click="openEditDialog(savedSearches[virtualItem.index])">
                  <Icon icon="tabler:pencil" class="saved-search-icon" />
                </button>
                <button class="saved-search-tool danger" type="button" :title="`Delete ${savedSearches[virtualItem.index].name}`" @click="openDeleteDialog(savedSearches[virtualItem.index])">
                  <Icon icon="tabler:trash" class="saved-search-icon" />
                </button>
              </div>
            </div>
          </div>
        </div>
        <span v-if="isFetchingNextSavedSearchPage" class="saved-search-loading-more">Loading saved searches...</span>
      </div>
      <div class="saved-search-group saved-search-actions">
        <button class="saved-search-tool" type="button" title="New saved search" @click="openCreateDialog(false)">
          <Icon icon="tabler:search-plus" class="saved-search-icon" />
        </button>
        <button class="saved-search-tool" type="button" title="New smart folder" @click="openCreateDialog(true)">
          <Icon icon="tabler:folder-plus" class="saved-search-icon" />
        </button>
      </div>
    </template>
    <Dialog v-model:open="dialogOpen" :title="dialogTitle" content-class="saved-search-dialog">
      <form class="saved-search-form" @submit.prevent="submitSavedSearch">
        <label class="saved-search-field">
          <span>Name</span>
          <input v-model="formValues.name" type="text" autocomplete="off" />
          <small v-if="errors.name">{{ errors.name }}</small>
        </label>
        <label class="saved-search-field">
          <span>Query</span>
          <input
            v-model="formValues.query"
            type="text"
            autocomplete="off"
            placeholder="Free text"
            @blur="normalizeQueryIntoBuilder(formValues.query)"
          />
          <small v-if="errors.query">{{ errors.query }}</small>
        </label>
        <div class="saved-search-rules-header">
          <span>Rules Builder</span>
        </div>
        <SavedSearchRuleGroupEditor :group="searchRuleTree" is-root />
        <p v-if="!ruleValidation.isValid" class="saved-search-rule-error">{{ ruleValidation.message }}</p>
        <label class="saved-search-field">
          <span>Description</span>
          <textarea v-model="formValues.description" rows="2" />
          <small v-if="errors.description">{{ errors.description }}</small>
        </label>
        <div class="saved-search-preset-row">
          <button
            v-for="preset in savedSearchPresetOptions"
            :key="preset.label"
            class="saved-search-preset"
            type="button"
            @click="applyPreset(preset)"
          >
            {{ preset.label }}
          </button>
        </div>
        <div class="saved-search-grid">
          <label class="saved-search-field">
            <span>Workflow</span>
            <select v-model="formValues.workflow_state">
              <option
                v-for="option in savedSearchWorkflowOptions"
                :key="option.label"
                :value="option.value"
              >
                {{ option.label }}
              </option>
            </select>
          </label>
          <label class="saved-search-field">
            <span>Local state</span>
            <select v-model="formValues.local_state">
              <option
                v-for="option in savedSearchLocalStateOptions"
                :key="option.value"
                :value="option.value"
              >
                {{ option.label }}
              </option>
            </select>
          </label>
        </div>
        <label class="saved-search-field">
          <span>Channel</span>
          <input
            v-model="formValues.channel_kind"
            type="text"
            autocomplete="off"
            list="saved-search-channel-options"
          />
          <datalist id="saved-search-channel-options">
            <option
              v-for="option in savedSearchChannelOptions"
              :key="option.label"
              :value="option.value"
            >
              {{ option.label }}
            </option>
          </datalist>
          <small v-if="errors.channel_kind">{{ errors.channel_kind }}</small>
        </label>
        <label class="saved-search-check">
          <input v-model="formValues.is_smart_folder" type="checkbox" />
          <span>Smart folder</span>
        </label>
        <div class="saved-search-rule-chips" aria-live="polite">
          <span
            v-for="chip in activeFilterChips"
            :key="`${chip.label}:${chip.value}`"
            class="saved-search-rule-chip"
          >
            <b>{{ chip.label }}</b>
            <span>{{ chip.value }}</span>
          </span>
        </div>
        <label class="saved-search-field">
          <span>Effective query</span>
          <output class="saved-search-effective-query">{{ effectiveQueryPreview || '—' }}</output>
        </label>
        <div class="saved-search-form-actions">
          <button class="saved-search-secondary" type="button" @click="dialogOpen = false">Cancel</button>
          <button class="saved-search-primary" type="submit" :disabled="isSaving || !ruleValidation.isValid">
            {{ isSaving ? 'Saving' : 'Save' }}
          </button>
        </div>
      </form>
    </Dialog>
    <Dialog
      v-model:open="deleteDialogOpen"
      :title="deleteCopy?.title ?? 'Delete saved search'"
      content-class="saved-search-delete-dialog"
    >
      <div class="saved-search-delete">
        <p>{{ deleteCopy?.message }}</p>
        <small v-if="deleteError">{{ deleteError }}</small>
        <div class="saved-search-form-actions">
          <button class="saved-search-secondary" type="button" @click="deleteDialogOpen = false">Cancel</button>
          <button class="saved-search-danger" type="button" :disabled="isDeleting" @click="confirmDeleteSavedSearch">
            {{ isDeleting ? 'Deleting' : deleteCopy?.confirmLabel ?? 'Delete' }}
          </button>
        </div>
      </div>
    </Dialog>
  </div>
</template>
