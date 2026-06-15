<script setup lang="ts">
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'

defineProps<{
  name: string
  nameError: string
  validationMessage: string
  canSave: boolean
  isSaving: boolean
  saveMode: 'new' | 'duplicate'
}>()

const emit = defineEmits<{
  cancel: []
  submit: []
  updateName: [value: string]
}>()
</script>

<template>
  <form class="template-save-form" @submit.prevent="emit('submit')">
    <span class="template-library-preview-meta">
      {{ saveMode === 'duplicate' ? 'Saving a new copy from the current compose content' : 'Saving the current compose content as a reusable template' }}
    </span>
    <label>
      <span>Template name</span>
      <input
        type="text"
        :value="name"
        :aria-invalid="Boolean(nameError)"
        @input="emit('updateName', ($event.target as HTMLInputElement).value)"
      />
    </label>
    <span v-if="nameError" class="template-error">
      {{ nameError }}
    </span>
    <span v-if="validationMessage" class="template-error">
      {{ validationMessage }}
    </span>
    <div class="template-save-actions">
      <Button type="button" variant="ghost" size="sm" @click="emit('cancel')">
        Cancel
      </Button>
      <Button
        type="submit"
        variant="secondary"
        size="sm"
        :disabled="!canSave"
        :loading="isSaving"
      >
        <Icon icon="tabler:device-floppy" size="16" />
        Save current
      </Button>
    </div>
  </form>
</template>
