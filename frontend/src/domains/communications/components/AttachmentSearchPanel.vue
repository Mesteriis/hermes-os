<script setup lang="ts">
import { computed, ref } from 'vue'
import { useForm } from 'vee-validate'
import { FlexRender, getCoreRowModel, useVueTable } from '@tanstack/vue-table'
import { useVirtualizer } from '@tanstack/vue-virtual'
import Icon from '../../../shared/ui/Icon.vue'
import { attachmentIcon } from '../stores/communications'
import { useAttachmentSearchQuery } from '../queries/useCommunicationsQuery'
import { useAttachmentSearchResultPrefetch } from '../queries/communicationPrefetch'
import {
  attachmentScanStatusOptions,
  attachmentSearchFormDefaults,
  attachmentSearchFormToRequest,
  attachmentSearchVeeValidationSchema,
  type AttachmentSearchFormValues
} from '../forms/attachmentSearchForm'
import type { AttachmentSearchRequest, AttachmentSearchResult } from '../types/attachments'
import { formatAttachmentSize, scanStatusClass } from './attachmentTable'
import {
  attachmentSearchTableColumns,
  attachmentSearchTableRowId
} from './attachmentSearchTable'
import './AttachmentSearchPanel.css'

const props = defineProps<{
  accountId: string | null
}>()

const isOpen = ref(false)
const hasSubmitted = ref(false)
const submittedRequest = ref<AttachmentSearchRequest>({ limit: 50 })
const parentRef = ref<HTMLDivElement | null>(null)
const prefetchAttachmentMessage = useAttachmentSearchResultPrefetch()

const {
  errors,
  handleSubmit,
  setFieldValue,
  values: formValues
} = useForm<AttachmentSearchFormValues>({
  validationSchema: attachmentSearchVeeValidationSchema,
  initialValues: attachmentSearchFormDefaults()
})

const {
  data: resultsData,
  error,
  fetchNextPage,
  hasNextPage,
  isFetchingNextPage,
  isLoading
} = useAttachmentSearchQuery(
  () => submittedRequest.value,
  () => isOpen.value && hasSubmitted.value
)

const results = computed(() => resultsData.value ?? [])
const errorMessage = computed(() => {
  if (!error.value) return ''
  return error.value instanceof Error ? error.value.message : 'Attachment search failed'
})

const table = useVueTable({
  get data() {
    return results.value
  },
  columns: attachmentSearchTableColumns,
  getCoreRowModel: getCoreRowModel(),
  getRowId: attachmentSearchTableRowId
})

const tableRows = computed(() => table.getRowModel().rows)
const virtualOptions = computed(() => ({
  count: tableRows.value.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 44,
  overscan: 8
}))
const virtualizer = useVirtualizer(virtualOptions)
const virtualRows = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())

const submitSearch = handleSubmit((values) => {
  submittedRequest.value = attachmentSearchFormToRequest(values, props.accountId)
  hasSubmitted.value = true
})

function toggleOpen() {
  isOpen.value = !isOpen.value
}

function updateSearchField(key: keyof AttachmentSearchFormValues, event: Event) {
  const input = event.target as HTMLInputElement | HTMLSelectElement
  setFieldValue(key, input.value)
}

function requestNextPage() {
  if (!hasNextPage.value || isFetchingNextPage.value) return
  void fetchNextPage()
}

function loadMore() {
  requestNextPage()
}

function handleResultPrefetch(result: AttachmentSearchResult) {
  void prefetchAttachmentMessage(result)
}

function handleResultsScroll() {
  const el = parentRef.value
  if (!el) return
  if (el.scrollTop + el.clientHeight >= el.scrollHeight - 180) {
    requestNextPage()
  }
}
</script>

<template>
  <section class="attachment-search-panel">
    <button class="attachment-search-toggle" type="button" :aria-expanded="isOpen" @click="toggleOpen">
      <span class="attachment-search-title">
        <Icon icon="tabler:paperclip" />
        Attachment search
      </span>
      <span class="attachment-search-count">{{ hasSubmitted ? `${results.length} results` : 'Metadata' }}</span>
    </button>

    <div v-if="isOpen" class="attachment-search-body">
      <form class="attachment-search-form" @submit.prevent="submitSearch">
        <label class="attachment-search-field">
          <span>Query</span>
          <input
            :value="formValues.query"
            type="text"
            autocomplete="off"
            placeholder="invoice, contract, pdf"
            @input="updateSearchField('query', $event)"
          />
          <small v-if="errors.query">{{ errors.query }}</small>
        </label>
        <label class="attachment-search-field">
          <span>Content type</span>
          <input
            :value="formValues.content_type"
            type="text"
            autocomplete="off"
            placeholder="application/pdf"
            @input="updateSearchField('content_type', $event)"
          />
          <small v-if="errors.content_type">{{ errors.content_type }}</small>
        </label>
        <label class="attachment-search-field">
          <span>Scan</span>
          <select :value="formValues.scan_status" @change="updateSearchField('scan_status', $event)">
            <option value="">Any</option>
            <option v-for="status in attachmentScanStatusOptions" :key="status" :value="status">{{ status }}</option>
          </select>
        </label>
        <button class="attachment-search-submit" type="submit" :disabled="isLoading">Search</button>
      </form>

      <p v-if="errorMessage" class="attachment-search-error">{{ errorMessage }}</p>
      <div v-else-if="hasSubmitted && results.length === 0 && !isLoading" class="attachment-search-empty">
        No attachment metadata found
      </div>
      <div
        v-else-if="results.length"
        ref="parentRef"
        class="attachment-search-table-shell"
        @scroll="handleResultsScroll"
      >
        <div class="attachment-search-grid" role="table">
          <div class="attachment-search-head" role="rowgroup">
            <div
              v-for="headerGroup in table.getHeaderGroups()"
              :key="headerGroup.id"
              class="attachment-search-row attachment-search-header-row"
              role="row"
            >
              <div
                v-for="header in headerGroup.headers"
                :key="header.id"
                class="attachment-search-cell attachment-search-cell--head"
                role="columnheader"
              >
                <FlexRender
                  v-if="!header.isPlaceholder"
                  :render="header.column.columnDef.header"
                  :props="header.getContext()"
                />
              </div>
            </div>
          </div>
          <div class="attachment-search-virtual" role="rowgroup" :style="{ height: `${totalSize}px` }">
            <div
              v-for="virtualRow in virtualRows"
              :key="String(virtualRow.key)"
              class="attachment-search-row attachment-search-result-row"
              role="row"
              :style="{
                transform: `translateY(${virtualRow.start}px)`,
                height: `${virtualRow.size}px`
              }"
              tabindex="0"
              @mouseenter="handleResultPrefetch(tableRows[virtualRow.index].original)"
              @focus="handleResultPrefetch(tableRows[virtualRow.index].original)"
            >
              <div
                v-for="cell in tableRows[virtualRow.index].getVisibleCells()"
                :key="cell.id"
                class="attachment-search-cell"
                role="cell"
              >
                <div v-if="cell.column.id === 'filename'" class="attachment-search-file">
                  <Icon :icon="attachmentIcon(tableRows[virtualRow.index].original.content_type)" />
                  <span class="attachment-search-name">
                    {{ tableRows[virtualRow.index].original.filename || 'Unnamed' }}
                  </span>
                </div>
                <div v-else-if="cell.column.id === 'message_subject'" class="attachment-search-message">
                  <Icon icon="tabler:mail" />
                  <span class="attachment-search-subject">
                    {{ tableRows[virtualRow.index].original.message_subject }}
                  </span>
                </div>
                <span v-else-if="cell.column.id === 'size'">
                  {{ formatAttachmentSize(tableRows[virtualRow.index].original.size_bytes) }}
                </span>
                <span
                  v-else-if="cell.column.id === 'scan_status'"
                  class="attachment-search-scan"
                  :class="scanStatusClass(tableRows[virtualRow.index].original.scan_status)"
                >
                  {{ tableRows[virtualRow.index].original.scan_status }}
                </span>
                <span v-else>{{ cell.getValue() }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <button
        v-if="hasSubmitted && hasNextPage"
        class="attachment-search-more"
        type="button"
        :disabled="isFetchingNextPage"
        @click="loadMore"
      >
        {{ isFetchingNextPage ? 'Loading...' : 'Load more' }}
      </button>
    </div>
  </section>
</template>
