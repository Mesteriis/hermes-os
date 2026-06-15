<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { Ref } from 'vue'
import { useForm } from 'vee-validate'
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import TemplateRecipientMappingPanel from './TemplateRecipientMappingPanel.vue'
import TemplateSaveForm from './TemplateSaveForm.vue'
import {
  useCreateRichTemplateMutation,
  useDeleteRichTemplateMutation,
  usePreviewRichTemplateMailMergeMutation,
  useRenderRichTemplateMutation,
  useRichTemplatesQuery
} from '../queries/useCommunicationsQuery'
import {
  missingTemplateVariables,
  parseTemplateMailMergePreviewRows,
  resolveTemplateVariableValues,
  stringifyTemplateMailMergePreviewRows,
  storedTemplateDiagnosticMessages,
  templateContentDiagnostics,
  templateDiagnosticsErrorMessage,
  templateFormDefaults,
  templateFormToInput,
  templateMergeErrorMessage,
  templateVeeValidationSchema,
  type TemplateFormValues
} from '../forms/templateForm'
import {
  applyTemplateRecipientMapping,
  buildTemplateRecipientPreviewRows,
  deriveTemplateLibraryCategories,
  filterTemplateLibraryTemplates,
  formatTemplateUpdatedLabel,
  inferRecipientVariableMapping,
  orderTemplateLibraryTemplates,
  recipientPreviewSummary,
  suggestTemplateSaveName,
  templateLibraryCategoryLabel,
  templateLibraryCategoryOptions,
  type TemplateLibraryCategory,
  type TemplateRecipientVariableMapping
} from './templateLibrary'
import type { EmailTemplate, RichTemplateMailMergePreviewResponse } from '../types/communications'
import './ComposeTemplatePicker.css'

const props = defineProps<{
  toText: string
  ccText: string
  bccText: string
  subject: string
  body: string
  bodyHtml: string | null
}>()

const emit = defineEmits<{
  apply: [payload: { subject: string; bodyHtml: string }]
  error: [message: string]
  saved: [name: string]
  deleted: [name: string]
}>()

const templatesQuery = useRichTemplatesQuery()
const createTemplateMutation = useCreateRichTemplateMutation()
const deleteTemplateMutation = useDeleteRichTemplateMutation()
const previewMailMergeMutation = usePreviewRichTemplateMailMergeMutation()
const renderTemplateMutation = useRenderRichTemplateMutation()
const selectedTemplateId = ref('')
const variableValues = ref<Record<string, string>>({})
const isSaveOpen = ref(false)
const deleteConfirmTemplateId = ref('')
const previewRowsText = ref('')
const previewError = ref('')
const previewResult = ref<RichTemplateMailMergePreviewResponse | null>(null)
const selectedCategory = ref<'all' | TemplateLibraryCategory>('all')
const saveMode = ref<'new' | 'duplicate'>('new')
const recipientVariableMapping = ref<TemplateRecipientVariableMapping>({
  toVariable: '',
  ccVariable: '',
  bccVariable: ''
})

const templates = computed(() => templatesQuery.data.value ?? [])
const isTemplatesLoading = computed(() => templatesQuery.isPending.value)
const isSavingTemplate = computed(() => createTemplateMutation.isPending.value)
const isDeletingTemplate = computed(() => deleteTemplateMutation.isPending.value)
const isPreviewingMailMerge = computed(() => previewMailMergeMutation.isPending.value)
const isRenderingTemplate = computed(() => renderTemplateMutation.isPending.value)
const selectedTemplate = computed(() => {
  return templates.value.find((template) => template.template_id === selectedTemplateId.value) ?? null
})
const templateVariables = computed(() => selectedTemplate.value?.variables ?? [])
const selectedTemplateDiagnostics = computed(() => {
  return storedTemplateDiagnosticMessages(selectedTemplate.value)
})
const selectedTemplateBlockingDiagnostics = computed(() => {
  return selectedTemplateDiagnostics.value.filter((message) => message.kind === 'error')
})
const selectedTemplateValidationMessage = computed(() => {
  const blocking = selectedTemplateBlockingDiagnostics.value[0]
  if (!blocking) return ''
  return `${blocking.label}: ${blocking.values.join(', ')}`
})
const missingMergeVariables = computed(() => {
  return missingTemplateVariables(templateVariables.value, variableValues.value)
})
const mergeValidationMessage = computed(() => {
  return templateMergeErrorMessage(missingMergeVariables.value)
})
const hasTemplateContent = computed(() => {
  return Boolean(props.subject.trim() || props.bodyHtml?.trim() || props.body.trim())
})
const saveDiagnostics = computed(() => {
  return templateContentDiagnostics(props.subject, props.bodyHtml ?? props.body)
})
const saveValidationMessage = computed(() => {
  return templateDiagnosticsErrorMessage(saveDiagnostics.value)
})
const canApplyTemplate = computed(() => {
  return Boolean(selectedTemplate.value)
    && !selectedTemplateValidationMessage.value
    && !mergeValidationMessage.value
    && !isRenderingTemplate.value
})
const canSaveTemplate = computed(() => {
  return hasTemplateContent.value && !saveValidationMessage.value && !isSavingTemplate.value
})
const canUpdateTemplate = computed(() => {
  return Boolean(selectedTemplate.value) && hasTemplateContent.value && !saveValidationMessage.value && !isSavingTemplate.value
})
const isDeleteArmed = computed(() => {
  return Boolean(selectedTemplate.value && deleteConfirmTemplateId.value === selectedTemplate.value.template_id)
})
const canDeleteTemplate = computed(() => {
  return Boolean(selectedTemplate.value) && !isDeletingTemplate.value
})
const templateLibraryQuery: Ref<string> = ref('')
const composeRecipientSummary = computed(() => recipientPreviewSummary({
  toText: props.toText,
  ccText: props.ccText,
  bccText: props.bccText
}))

const filteredTemplates = computed(() => {
  return filterTemplateLibraryTemplates(
    templates.value,
    templateLibraryQuery.value,
    selectedCategory.value
  )
})
const orderedTemplates = computed(() => orderTemplateLibraryTemplates(filteredTemplates.value))

const selectedTemplateSubjectPreview = computed(() => selectedTemplate.value?.subject_template ?? '')
const selectedTemplateBodyPreview = computed(() => selectedTemplate.value?.body_template ?? '')
const selectedTemplateCategories = computed(() => (
  selectedTemplate.value ? deriveTemplateLibraryCategories(selectedTemplate.value) : []
))

const {
  errors: saveFormErrors,
  handleSubmit,
  resetForm,
  setFieldValue,
  values: saveFormValues
} = useForm<TemplateFormValues>({
  validationSchema: templateVeeValidationSchema,
  initialValues: templateFormDefaults()
})

const templateVariableDefaultsContext = computed(() => ({
  toText: props.toText,
  ccText: props.ccText,
  bccText: props.bccText,
  subject: props.subject,
  body: props.body
}))

watch(
  selectedTemplate,
  (template, previousTemplate) => {
    const isSameTemplate = Boolean(template && previousTemplate && template.template_id === previousTemplate.template_id)
    variableValues.value = resolveTemplateVariableValues(
      template,
      variableValues.value,
      templateVariableDefaultsContext.value,
      {
        preserveExisting: isSameTemplate
      }
    )
    if (!isSameTemplate) {
      recipientVariableMapping.value = inferRecipientVariableMapping(template?.variables ?? [])
      previewRowsText.value = stringifyTemplateMailMergePreviewRows(buildDefaultPreviewRows(template))
      previewError.value = ''
      previewResult.value = null
    }
    deleteConfirmTemplateId.value = ''
  },
  { immediate: true }
)
watch(
  orderedTemplates,
  (items) => {
    if (!items.length) {
      selectedTemplateId.value = ''
      return
    }
    if (!items.some((template) => template.template_id === selectedTemplateId.value)) {
      selectedTemplateId.value = items[0].template_id
    }
  },
  { immediate: true }
)

function updateVariable(variable: string, value: string): void {
  variableValues.value = {
    ...variableValues.value,
    [variable]: value
  }
}

function templateDiagnosticCount(template: EmailTemplate): number {
  return storedTemplateDiagnosticMessages(template).length
}

function templateUpdatedLabel(template: EmailTemplate): string {
  return formatTemplateUpdatedLabel(template.updated_at)
}

async function applyTemplate(): Promise<void> {
  const template = selectedTemplate.value
  if (!template) return
  if (selectedTemplateValidationMessage.value) {
    emit('error', selectedTemplateValidationMessage.value)
    return
  }
  if (mergeValidationMessage.value) {
    emit('error', mergeValidationMessage.value)
    return
  }

  try {
    const result = await renderTemplateMutation.mutateAsync({
      template_id: template.template_id,
      variables: variableValues.value
    })
    if (result.rendered.malformed_placeholders.length) {
      emit('error', `Fix malformed template placeholders: ${result.rendered.malformed_placeholders.join(', ')}`)
      return
    }
    if (result.rendered.unresolved_variables.length) {
      emit('error', templateMergeErrorMessage(result.rendered.unresolved_variables))
      return
    }
    emit('apply', {
      subject: result.rendered.subject,
      bodyHtml: result.rendered.body
    })
  } catch (error) {
    emit('error', error instanceof Error ? error.message : 'Template render failed')
  }
}

function openSaveTemplate(mode: 'new' | 'duplicate'): void {
  saveMode.value = mode
  isSaveOpen.value = true
  resetForm({
    values: {
      name: suggestTemplateSaveName(props.subject, selectedTemplate.value?.name ?? '', {
        duplicate: mode === 'duplicate'
      })
    }
  })
}

function closeSaveTemplate(): void {
  saveMode.value = 'new'
  isSaveOpen.value = false
  resetForm({ values: templateFormDefaults() })
}

function clearTemplateLibraryQuery(): void {
  templateLibraryQuery.value = ''
}

function buildDefaultPreviewRows(template: EmailTemplate | null): Array<{ row_id: string; variables: Record<string, string> }> {
  if (!template) return []
  const values = resolveTemplateVariableValues(
    template,
    variableValues.value,
    templateVariableDefaultsContext.value,
    { preserveExisting: true }
  )
  if (!Object.keys(values).length) return []
  return [{ row_id: 'row-1', variables: values }]
}

function applyRecipientMapping(): void {
  variableValues.value = applyTemplateRecipientMapping(variableValues.value, recipientVariableMapping.value, {
    toText: props.toText,
    ccText: props.ccText,
    bccText: props.bccText
  })
}

function buildPreviewRowsFromRecipients(): void {
  const rows = buildTemplateRecipientPreviewRows(
    templateVariables.value,
    recipientVariableMapping.value,
    {
      toText: props.toText,
      ccText: props.ccText,
      bccText: props.bccText
    },
    variableValues.value
  )
  if (!rows.length) {
    previewError.value = 'Map a To variable and add at least one To recipient to build preview rows'
    return
  }
  previewRowsText.value = stringifyTemplateMailMergePreviewRows(rows)
  previewError.value = ''
  previewResult.value = null
}

async function previewMailMerge(): Promise<void> {
  const template = selectedTemplate.value
  if (!template) return
  if (selectedTemplateValidationMessage.value) {
    emit('error', selectedTemplateValidationMessage.value)
    return
  }

  previewError.value = ''
  previewResult.value = null
  try {
    const rows = parseTemplateMailMergePreviewRows(previewRowsText.value)
    if (!rows.length) {
      previewError.value = 'Add at least one preview row'
      return
    }
    previewResult.value = await previewMailMergeMutation.mutateAsync({
      template_id: template.template_id,
      rows
    })
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Mail merge preview failed'
    previewError.value = message
    emit('error', message)
  }
}

const saveCurrentTemplate = handleSubmit(async (values) => {
  if (!hasTemplateContent.value) {
    emit('error', 'Add a subject or body before saving a template')
    return
  }
  if (saveValidationMessage.value) {
    emit('error', saveValidationMessage.value)
    return
  }

  try {
    const result = await createTemplateMutation.mutateAsync(templateFormToInput(values, {
      subject: props.subject,
      body: props.body,
      bodyHtml: props.bodyHtml
    }))
    selectedTemplateId.value = result.template.template_id
    closeSaveTemplate()
    emit('saved', result.template.name)
  } catch (error) {
    emit('error', error instanceof Error ? error.message : 'Template save failed')
  }
})

async function updateSelectedTemplate(): Promise<void> {
  const template = selectedTemplate.value
  if (!template || !hasTemplateContent.value) return
  if (saveValidationMessage.value) {
    emit('error', saveValidationMessage.value)
    return
  }

  try {
    const result = await createTemplateMutation.mutateAsync(templateFormToInput(
      { name: template.name },
      {
        templateId: template.template_id,
        subject: props.subject,
        body: props.body,
        bodyHtml: props.bodyHtml
      }
    ))
    selectedTemplateId.value = result.template.template_id
    emit('saved', result.template.name)
  } catch (error) {
    emit('error', error instanceof Error ? error.message : 'Template update failed')
  }
}

async function deleteSelectedTemplate(): Promise<void> {
  const template = selectedTemplate.value
  if (!template) return
  if (!isDeleteArmed.value) {
    deleteConfirmTemplateId.value = template.template_id
    return
  }

  try {
    await deleteTemplateMutation.mutateAsync(template.template_id)
    selectedTemplateId.value = ''
    variableValues.value = {}
    deleteConfirmTemplateId.value = ''
    emit('deleted', template.name)
  } catch (error) {
    emit('error', error instanceof Error ? error.message : 'Template delete failed')
  }
}
</script>

<template>
  <section class="compose-template-picker" aria-label="Email templates">
    <div class="template-toolbar">
      <div class="template-toolbar-main">
        <Button
          variant="secondary"
          size="sm"
          :disabled="!canApplyTemplate"
          :loading="isRenderingTemplate"
          @click="applyTemplate"
        >
          <Icon icon="tabler:template" size="16" />
          Apply
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :disabled="isSavingTemplate"
          @click="openSaveTemplate('new')"
        >
          <Icon icon="tabler:device-floppy" size="16" />
          Save
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :disabled="!selectedTemplate || !hasTemplateContent || isSavingTemplate"
          @click="openSaveTemplate('duplicate')"
        >
          <Icon icon="tabler:copy-plus" size="16" />
          Save copy
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :disabled="!canUpdateTemplate"
          :loading="isSavingTemplate"
          @click="updateSelectedTemplate"
        >
          <Icon icon="tabler:refresh" size="16" />
          Update
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :disabled="!canDeleteTemplate"
          :loading="isDeletingTemplate"
          @click="deleteSelectedTemplate"
        >
          <Icon icon="tabler:trash" size="16" />
          {{ isDeleteArmed ? 'Confirm' : 'Delete' }}
        </Button>
      </div>
      <div class="template-toolbar-side">
        <label class="template-library-search">
          <span>Search templates</span>
          <div class="template-library-search-row">
            <input
              v-model="templateLibraryQuery"
              type="text"
              placeholder="Search by name, subject, or variables"
            />
            <Button
              v-if="templateLibraryQuery"
              variant="ghost"
              size="sm"
              type="button"
              @click="clearTemplateLibraryQuery"
            >
              <Icon icon="tabler:x" size="14" />
            </Button>
          </div>
        </label>
        <label class="template-category-filter">
          <span>Categories</span>
          <div class="template-category-filter-row">
            <button
              class="template-category-chip"
              :class="{ active: selectedCategory === 'all' }"
              type="button"
              @click="selectedCategory = 'all'"
            >
              All
            </button>
            <button
              v-for="category in templateLibraryCategoryOptions"
              :key="category.value"
              class="template-category-chip"
              :class="{ active: selectedCategory === category.value }"
              type="button"
              @click="selectedCategory = category.value"
            >
              {{ category.label }}
            </button>
          </div>
        </label>
      </div>
    </div>

    <div class="template-library-surface">
      <label class="template-library-empty">
        {{ isTemplatesLoading ? 'Loading templates...' : 'Select template' }}
      </label>
      <div class="template-library-list">
        <button
          v-for="template in orderedTemplates"
          :key="template.template_id"
          class="template-library-item"
          :class="{ active: template.template_id === selectedTemplateId }"
          type="button"
          @click="selectedTemplateId = template.template_id"
        >
          <span class="template-library-item-title">{{ template.name }}</span>
          <span class="template-library-item-meta">
            <span class="template-library-item-count">{{ template.variables.length }} vars</span>
            <span class="template-library-item-updated">{{ templateUpdatedLabel(template) }}</span>
            <span v-if="templateDiagnosticCount(template)" class="template-library-item-diagnostics">
              {{ templateDiagnosticCount(template) }} issues
            </span>
            <span
              v-for="category in deriveTemplateLibraryCategories(template)"
              :key="`${template.template_id}:${category}`"
              class="template-library-tag"
            >
              {{ templateLibraryCategoryLabel(category) }}
            </span>
          </span>
        </button>
        <span v-if="!orderedTemplates.length" class="template-library-empty-state">
          No matching templates
        </span>
      </div>
      <div class="template-library-preview">
        <h4>{{ selectedTemplate?.name ?? 'Template preview' }}</h4>
        <span v-if="selectedTemplate" class="template-library-preview-meta">
          {{ selectedTemplate.variables.length }} variables · Updated {{ templateUpdatedLabel(selectedTemplate) }}
        </span>
        <div v-if="selectedTemplateCategories.length" class="template-library-preview-categories">
          <span
            v-for="category in selectedTemplateCategories"
            :key="`preview:${category}`"
            class="template-library-tag"
          >
            {{ templateLibraryCategoryLabel(category) }}
          </span>
        </div>
        <label>
          <span>Subject</span>
          <pre>{{ selectedTemplateSubjectPreview || '—' }}</pre>
        </label>
        <label>
          <span>Body</span>
          <pre>{{ selectedTemplateBodyPreview || '—' }}</pre>
        </label>
      </div>
    </div>

    <div v-if="selectedTemplateDiagnostics.length" class="template-diagnostics" aria-live="polite">
      <div
        v-for="message in selectedTemplateDiagnostics"
        :key="`${message.label}:${message.values.join('|')}`"
        class="template-diagnostic"
        :data-kind="message.kind"
      >
        <Icon
          :icon="message.kind === 'error' ? 'tabler:alert-triangle' : 'tabler:info-circle'"
          size="15"
        />
        <span>{{ message.label }}: {{ message.values.join(', ') }}</span>
      </div>
    </div>

    <div v-if="templateVariables.length" class="template-variables">
      <label
        v-for="variable in templateVariables"
        :key="variable"
      >
        <span>{{ variable }}</span>
        <input
          type="text"
          :value="variableValues[variable] ?? ''"
          :aria-invalid="missingMergeVariables.includes(variable)"
          @input="updateVariable(variable, ($event.target as HTMLInputElement).value)"
        />
      </label>
      <span v-if="mergeValidationMessage" class="template-error">
        {{ mergeValidationMessage }}
      </span>
    </div>

    <TemplateRecipientMappingPanel
      v-if="selectedTemplate && templateVariables.length"
      :template-variables="templateVariables"
      :mapping="recipientVariableMapping"
      :summary="composeRecipientSummary"
      @update:mapping="recipientVariableMapping = $event"
      @fill="applyRecipientMapping"
      @build-preview="buildPreviewRowsFromRecipients"
    />

    <div v-if="selectedTemplate" class="template-mail-merge-preview">
      <div class="template-mail-merge-preview-header">
        <h4>Mail merge preview</h4>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          :disabled="isPreviewingMailMerge"
          :loading="isPreviewingMailMerge"
          @click="previewMailMerge"
        >
          <Icon icon="tabler:table-share" size="16" />
          Preview rows
        </Button>
      </div>
      <label>
        <span>Rows JSON</span>
        <textarea
          v-model="previewRowsText"
          rows="8"
          spellcheck="false"
          placeholder='[{"row_id":"row-1","variables":{"recipient":"alex@example.com"}}]'
        />
      </label>
      <span v-if="previewError" class="template-error">{{ previewError }}</span>
      <div v-if="previewResult" class="template-mail-merge-preview-result">
        <span class="template-mail-merge-preview-summary">
          {{ previewResult.ready_count }} ready · {{ previewResult.blocked_count }} blocked · {{ previewResult.row_count }} rows
        </span>
        <div
          v-for="item in previewResult.items"
          :key="item.row_id"
          class="template-mail-merge-preview-item"
          :data-ready="item.ready"
        >
          <div class="template-mail-merge-preview-row-header">
            <strong>{{ item.row_id }}</strong>
            <span>{{ item.ready ? 'Ready' : 'Blocked' }}</span>
          </div>
          <label>
            <span>Subject</span>
            <pre>{{ item.rendered.subject || '—' }}</pre>
          </label>
          <label>
            <span>Body</span>
            <pre>{{ item.rendered.body || '—' }}</pre>
          </label>
          <span
            v-if="item.rendered.unresolved_variables.length || item.rendered.malformed_placeholders.length"
            class="template-error"
          >
            {{
              item.rendered.malformed_placeholders.length
                ? `Malformed: ${item.rendered.malformed_placeholders.join(', ')}`
                : `Missing: ${item.rendered.unresolved_variables.join(', ')}`
            }}
          </span>
        </div>
      </div>
    </div>

    <TemplateSaveForm
      v-if="isSaveOpen"
      :name="saveFormValues.name"
      :name-error="saveFormErrors.name ?? ''"
      :validation-message="saveValidationMessage"
      :can-save="canSaveTemplate"
      :is-saving="isSavingTemplate"
      :save-mode="saveMode"
      @cancel="closeSaveTemplate"
      @submit="saveCurrentTemplate"
      @update-name="setFieldValue('name', $event)"
    />
  </section>
</template>
