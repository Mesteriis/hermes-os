<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useForm } from 'vee-validate'
import { useVirtualizer } from '@tanstack/vue-virtual'
import Dialog from '../../../shared/ui/Dialog.vue'
import Icon from '../../../shared/ui/Icon.vue'
import {
  useCopyMessageToFolderMutation,
  useCreateMailFolderMutation,
  useDeleteMailFolderMutation,
  useMailFoldersQuery,
  useMoveMessageToFolderMutation,
  useUpdateMailFolderMutation
} from '../queries/useCommunicationsQuery'
import {
  composeMailFolderName,
  mailFolderDeleteDialogCopy,
  mailFolderFormDefaults,
  mailFolderFormToInput,
  mailFolderParentPathOptions,
  mailFolderMessageCountLabel,
  mailFolderVeeValidationSchema,
  splitMailFolderName,
  validateMailFolderParentPath,
  type MailFolderFormValues
} from '../forms/mailFolderForm'
import {
  MAIL_MESSAGE_DRAG_TYPE,
  hasCommunicationMessageDragType,
  parseCommunicationMessageDragPayload
} from './mailDragDrop'
import './MailFolderStrip.css'
import {
  createChildFolderDraft,
  mailFolderHierarchyDeleteImpact,
  mailFolderColorClass,
  orderMailFolderDisplayRows,
  type MailFolderDisplayRow
} from './mailFolderPresentation'
import { useMailFolderReorder } from './useMailFolderReorder'
import type { MailFolder } from '../types/folders'

const props = defineProps<{
  accountId: string | null
  activeId: string
}>()
const emit = defineEmits<{
  select: [folderId: string]
  deleted: [folder: MailFolder]
}>()

const {
  data: folderData,
  fetchNextPage,
  hasNextPage,
  isFetchingNextPage,
  isLoading
} = useMailFoldersQuery(() => props.accountId || undefined)
const folders = computed(() => folderData.value ?? [])
const createMutation = useCreateMailFolderMutation()
const updateMutation = useUpdateMailFolderMutation()
const deleteMutation = useDeleteMailFolderMutation()
const copyMessageMutation = useCopyMessageToFolderMutation()
const moveMessageMutation = useMoveMessageToFolderMutation()
const dialogOpen = ref(false)
const editingFolder = ref<MailFolder | null>(null)
const deleteDialogOpen = ref(false)
const deletingFolder = ref<MailFolder | null>(null)
const deleteError = ref('')
const folderPathError = ref('')
const dropTargetId = ref('')
const dropStatus = ref('')
const dropError = ref('')
const folderVirtualScrollRef = ref<HTMLDivElement | null>(null)
const parentPath = ref('')
const leafName = ref('')
const isSaving = computed(() => createMutation.isPending.value || updateMutation.isPending.value)
const isDeleting = computed(() => deleteMutation.isPending.value)
const isDropping = computed(() =>
  copyMessageMutation.isPending.value || moveMessageMutation.isPending.value || folderReorder.isReordering.value
)
const dialogTitle = computed(() => editingFolder.value ? 'Edit folder' : 'New folder')
const folderDialogDescription = computed(() =>
  editingFolder.value
    ? 'Update the local folder name, color and ordering.'
    : 'Create a local folder for the selected mail account.'
)
const deleteCopy = computed(() => {
  return deletingFolder.value ? mailFolderDeleteDialogCopy(deletingFolder.value) : null
})
const deleteImpact = computed(() => {
  return deletingFolder.value
    ? mailFolderHierarchyDeleteImpact(orderedFolders.value, deletingFolder.value.folder_id)
    : { descendantCount: 0, descendantLeafNames: [] }
})
const deleteDialogDescription = computed(() =>
  deleteCopy.value?.message ?? 'Confirm local folder deletion.'
)
const folderVirtualOptions = computed(() => ({
  count: folders.value.length,
  getScrollElement: () => folderVirtualScrollRef.value,
  estimateSize: () => 216,
  horizontal: true,
  overscan: 6
}))
const folderVirtualizer = useVirtualizer(folderVirtualOptions)
const virtualFolders = computed(() => folderVirtualizer.value.getVirtualItems())
const folderVirtualTotalSize = computed(() => folderVirtualizer.value.getTotalSize())

const {
  errors,
  handleSubmit,
  resetForm,
  setFieldValue,
  values: formValues
} = useForm<MailFolderFormValues>({
  validationSchema: mailFolderVeeValidationSchema,
  initialValues: mailFolderFormDefaults()
})

const folderRows = computed<MailFolderDisplayRow[]>(() => orderMailFolderDisplayRows(folders.value))
const orderedFolders = computed(() => folderRows.value.map((row) => row.folder))
const folderReorder = useMailFolderReorder(orderedFolders, updateMutation.mutateAsync)
const folderPathPreview = computed(() => composeMailFolderName(parentPath.value, leafName.value))
const parentPathOptions = computed(() => mailFolderParentPathOptions(orderedFolders.value, editingFolder.value))
const parentPathValidationMessage = computed(() => validateMailFolderParentPath(parentPath.value, editingFolder.value))

const folderIndentUnit = 0.75

function folderIndent(folder: MailFolderDisplayRow): string {
  return `${Math.min(folder.depth, 8) * folderIndentUnit}rem`
}

watch(dialogOpen, (open) => {
  if (!open) {
    editingFolder.value = null
    folderPathError.value = ''
  }
})
watch(deleteDialogOpen, (open) => {
  if (!open) deletingFolder.value = null
})
watch([parentPath, leafName], () => {
  setFieldValue('name', folderPathPreview.value)
  if (folderPathError.value && !parentPathValidationMessage.value) {
    folderPathError.value = ''
  }
})

function openCreateDialog() {
  editingFolder.value = null
  resetForm({ values: mailFolderFormDefaults() })
  syncFolderNameParts('')
  setFieldValue('sort_order', 0)
  dialogOpen.value = true
}

function openCreateChildDialog(folder: MailFolder) {
  editingFolder.value = null
  const draft = createChildFolderDraft(folder)
  resetForm({
    values: mailFolderFormDefaults({
      ...folder,
      folder_id: '',
      name: '',
      description: null,
      message_count: 0,
      sort_order: draft.sortOrder
    })
  })
  parentPath.value = draft.parentPath
  leafName.value = ''
  setFieldValue('name', '')
  setFieldValue('sort_order', draft.sortOrder)
  folderPathError.value = ''
  dialogOpen.value = true
}

function openEditDialog(folder: MailFolder) {
  editingFolder.value = folder
  resetForm({ values: mailFolderFormDefaults(folder) })
  syncFolderNameParts(folder.name)
  dialogOpen.value = true
}

function openDeleteDialog(folder: MailFolder) {
  deletingFolder.value = folder
  deleteError.value = ''
  deleteDialogOpen.value = true
}

const submitFolder = handleSubmit(async (values) => {
  if (parentPathValidationMessage.value) {
    folderPathError.value = parentPathValidationMessage.value
    return
  }
  const request = mailFolderFormToInput(values, props.accountId)
  if (editingFolder.value) {
    await updateMutation.mutateAsync({
      folderId: editingFolder.value.folder_id,
      request
    })
  } else {
    await createMutation.mutateAsync(request)
  }
  dialogOpen.value = false
})

async function confirmDeleteFolder() {
  const folder = deletingFolder.value
  if (!folder) return
  deleteError.value = ''
  try {
    await deleteMutation.mutateAsync(folder.folder_id)
    if (folder.folder_id === props.activeId) emit('deleted', folder)
    deleteDialogOpen.value = false
  } catch (error) {
    deleteError.value = error instanceof Error ? error.message : 'Folder deletion failed'
  }
}

function updateField(key: keyof MailFolderFormValues, event: Event) {
  const input = event.target as HTMLInputElement
  setFieldValue(key, key === 'sort_order' ? Number(input.value) : input.value)
}

function updateParentPath(event: Event) {
  parentPath.value = (event.target as HTMLInputElement).value
}

function updateLeafName(event: Event) {
  leafName.value = (event.target as HTMLInputElement).value
}

function syncFolderNameParts(name: string) {
  const parts = splitMailFolderName(name)
  parentPath.value = parts.parentPath
  leafName.value = parts.leafName
  setFieldValue('name', composeMailFolderName(parts.parentPath, parts.leafName))
  folderPathError.value = ''
}

function handleFolderDragOver(event: DragEvent) {
  if (!event.dataTransfer || isDropping.value) return
  if (folderReorder.canHandleDragOver(event)) {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
    return
  }
  if (!hasCommunicationMessageDragType(event.dataTransfer.types)) return
  event.preventDefault()
  event.dataTransfer.dropEffect = event.altKey ? 'copy' : 'move'
}

function handleFolderVirtualScroll() {
  const el = folderVirtualScrollRef.value
  if (!el || !hasNextPage.value || isFetchingNextPage.value) return
  if (el.scrollLeft + el.clientWidth >= el.scrollWidth - 320) {
    void fetchNextPage()
  }
}

async function handleFolderDrop(event: DragEvent, folder: MailFolder) {
  if (!event.dataTransfer || isDropping.value) return
  if (await folderReorder.handleDrop(event, folder)) {
    dropStatus.value = ''
    dropError.value = ''
    return
  }
  const payload = parseCommunicationMessageDragPayload(event.dataTransfer.getData(MAIL_MESSAGE_DRAG_TYPE))
  if (!payload) return

  const operation = event.altKey ? 'copy' : 'move'
  const mutation = operation === 'copy' ? copyMessageMutation : moveMessageMutation
  dropTargetId.value = folder.folder_id
  dropStatus.value = ''
  dropError.value = ''

  try {
    await Promise.all(payload.message_ids.map((messageId) =>
      mutation.mutateAsync({ folderId: folder.folder_id, messageId })
    ))
    const verb = operation === 'copy' ? 'Copied' : 'Moved'
    dropStatus.value = `${verb} ${payload.message_ids.length} message${payload.message_ids.length === 1 ? '' : 's'} to ${folder.name}`
  } catch (error) {
    dropError.value = error instanceof Error ? error.message : 'Folder drop failed'
  } finally {
    dropTargetId.value = ''
  }
}
</script>

<template>
  <div class="mail-folder-strip">
    <div v-if="isLoading" class="mail-folder-skeleton" />
    <template v-else>
      <span v-if="folders.length" class="mail-folder-label">Folders</span>
      <div
        v-if="folders.length"
        ref="folderVirtualScrollRef"
        class="mail-folder-virtual-scroll"
        @scroll="handleFolderVirtualScroll"
      >
        <div
          class="mail-folder-virtual-track"
          :style="{ width: `${folderVirtualTotalSize}px` }"
        >
          <div
            v-for="virtualFolder in virtualFolders"
            :key="String(virtualFolder.key)"
            class="mail-folder-virtual-row"
            :style="{
              width: `${virtualFolder.size}px`,
              transform: `translateX(${virtualFolder.start}px)`
            }"
          >
            <div
              class="mail-folder-item"
              :class="{ active: folderRows[virtualFolder.index].folder.folder_id === activeId, dropping: dropTargetId === folderRows[virtualFolder.index].folder.folder_id || folderReorder.targetId.value === folderRows[virtualFolder.index].folder.folder_id, reordering: folderReorder.sourceId.value === folderRows[virtualFolder.index].folder.folder_id }"
              @dragover="handleFolderDragOver"
              @drop.prevent="handleFolderDrop($event, folderRows[virtualFolder.index].folder)"
            >
              <span class="mail-folder-indent" :style="{ width: folderIndent(folderRows[virtualFolder.index]) }" />
              <button
                class="mail-folder-reorder"
                type="button"
                draggable="true"
                :title="`Reorder ${folderRows[virtualFolder.index].folder.name}`"
                @dragstart="folderReorder.handleDragStart($event, folderRows[virtualFolder.index].folder)"
                @dragend="folderReorder.handleDragEnd"
              >
                <Icon icon="tabler:grip-vertical" class="mail-folder-icon" />
              </button>
              <button
                class="mail-folder-select"
                type="button"
                :aria-pressed="folderRows[virtualFolder.index].folder.folder_id === activeId"
                :title="`Show ${folderRows[virtualFolder.index].folder.name}`"
                @click="emit('select', folderRows[virtualFolder.index].folder.folder_id)"
              >
                <span :class="['mail-folder-color', mailFolderColorClass(folderRows[virtualFolder.index].folder.color)]" />
                <span class="mail-folder-name">{{ folderRows[virtualFolder.index].leafName }}</span>
                <span v-if="folderRows[virtualFolder.index].pathPrefix" class="mail-folder-path">{{ folderRows[virtualFolder.index].pathPrefix }}</span>
                <span class="mail-folder-count">{{ mailFolderMessageCountLabel(folderRows[virtualFolder.index].folder) }}</span>
              </button>
              <button class="mail-folder-tool" type="button" :title="`Add child folder under ${folderRows[virtualFolder.index].folder.name}`" @click="openCreateChildDialog(folderRows[virtualFolder.index].folder)">
                <Icon icon="tabler:folder-plus" class="mail-folder-icon" />
              </button>
              <button class="mail-folder-tool" type="button" :title="`Edit ${folderRows[virtualFolder.index].folder.name}`" @click="openEditDialog(folderRows[virtualFolder.index].folder)">
                <Icon icon="tabler:pencil" class="mail-folder-icon" />
              </button>
              <button class="mail-folder-tool danger" type="button" :title="`Delete ${folderRows[virtualFolder.index].folder.name}`" @click="openDeleteDialog(folderRows[virtualFolder.index].folder)">
                <Icon icon="tabler:trash" class="mail-folder-icon" />
              </button>
            </div>
          </div>
        </div>
      </div>
      <button class="mail-folder-tool" type="button" title="New folder" @click="openCreateDialog">
        <Icon icon="tabler:folder-plus" class="mail-folder-icon" />
      </button>
      <span v-if="dropStatus" class="mail-folder-drop-status">{{ dropStatus }}</span>
      <span v-if="isFetchingNextPage" class="mail-folder-drop-status">Loading folders...</span>
      <span v-if="folderReorder.status.value" class="mail-folder-drop-status">{{ folderReorder.status.value }}</span>
      <span v-if="dropError" class="mail-folder-error">{{ dropError }}</span>
      <span v-if="folderReorder.error.value" class="mail-folder-error">{{ folderReorder.error.value }}</span>
    </template>

    <Dialog
      v-model:open="dialogOpen"
      :title="dialogTitle"
      :description="folderDialogDescription"
      content-class="mail-folder-dialog"
    >
      <form class="mail-folder-form" @submit.prevent="submitFolder">
        <label class="mail-folder-field">
          <span>Parent folder</span>
          <input
            type="text"
            list="mail-folder-parent-path-options"
            :value="parentPath"
            placeholder="Top level"
            @input="updateParentPath"
          />
        </label>
        <datalist id="mail-folder-parent-path-options">
          <option v-for="option in parentPathOptions" :key="option" :value="option" />
        </datalist>
        <label class="mail-folder-field">
          <span>Folder name</span>
          <input
            type="text"
            :value="leafName"
            placeholder="Client projects"
            @input="updateLeafName"
          />
          <span v-if="errors.name" class="mail-folder-error">{{ errors.name }}</span>
        </label>
        <label class="mail-folder-field">
          <span>Full path</span>
          <output class="mail-folder-path-preview">{{ folderPathPreview || 'Top-level folder' }}</output>
          <span v-if="folderPathError" class="mail-folder-error">{{ folderPathError }}</span>
        </label>
        <label class="mail-folder-field">
          <span>Description</span>
          <textarea
            :value="formValues.description"
            rows="3"
            placeholder="Optional folder description"
            @input="updateField('description', $event)"
          />
          <span v-if="errors.description" class="mail-folder-error">{{ errors.description }}</span>
        </label>
        <div class="mail-folder-form-row">
          <label class="mail-folder-field">
            <span>Color</span>
            <input
              type="text"
              :value="formValues.color"
              placeholder="#3b82f6"
              @input="updateField('color', $event)"
            />
            <span v-if="errors.color" class="mail-folder-error">{{ errors.color }}</span>
          </label>
          <label class="mail-folder-field">
            <span>Sort</span>
            <input
              type="number"
              :value="formValues.sort_order"
              min="0"
              @input="updateField('sort_order', $event)"
            />
            <span v-if="errors.sort_order" class="mail-folder-error">{{ errors.sort_order }}</span>
          </label>
        </div>
      </form>
      <template #footer>
        <button class="mail-folder-dialog-button" type="button" @click="dialogOpen = false">Cancel</button>
        <button class="mail-folder-dialog-button primary" type="button" :disabled="isSaving" @click="submitFolder">
          {{ isSaving ? 'Saving...' : 'Save' }}
        </button>
      </template>
    </Dialog>

    <Dialog
      v-model:open="deleteDialogOpen"
      :title="deleteCopy?.title"
      :description="deleteDialogDescription"
      content-class="mail-folder-dialog"
    >
      <div v-if="deleteImpact.descendantCount" class="mail-folder-delete-impact">
        <p>
          {{ deleteImpact.descendantCount }} child path{{ deleteImpact.descendantCount === 1 ? '' : 's' }} will keep their existing full names.
        </p>
        <p v-if="deleteImpact.descendantLeafNames.length" class="mail-folder-delete-impact-note">
          Affected: {{ deleteImpact.descendantLeafNames.join(', ') }}
        </p>
      </div>
      <p v-if="deleteError" class="mail-folder-error">{{ deleteError }}</p>
      <template #footer>
        <button class="mail-folder-dialog-button" type="button" @click="deleteDialogOpen = false">Cancel</button>
        <button class="mail-folder-dialog-button danger" type="button" :disabled="isDeleting" @click="confirmDeleteFolder">
          {{ isDeleting ? 'Deleting...' : deleteCopy?.confirmLabel }}
        </button>
      </template>
    </Dialog>
  </div>
</template>
