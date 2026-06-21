<script setup lang="ts">
import { computed, watch } from 'vue'
import { useForm } from 'vee-validate'
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import {
  syncSettingsFormDefaults,
  syncSettingsFormToUpdate,
  syncSettingsVeeValidationSchema,
  type SyncSettingsFormValues
} from '../forms/syncSettingsForm'
import type {
  MailSyncSettings,
  MailSyncSettingsUpdate
} from '../../../shared/mailSync/types'

const props = defineProps<{
  settings: MailSyncSettings | null
  isLoading: boolean
  isSaving: boolean
}>()

const emit = defineEmits<{
  update: [settings: MailSyncSettingsUpdate]
}>()

const {
  errors,
  handleSubmit,
  setFieldValue,
  setValues,
  values: formValues
} = useForm<SyncSettingsFormValues>({
  validationSchema: syncSettingsVeeValidationSchema,
  initialValues: syncSettingsFormDefaults(props.settings)
})

const isDisabled = computed(() => props.isLoading || props.isSaving || !props.settings)
const syncStateLabel = computed(() => (formValues.sync_enabled ? 'Enabled' : 'Paused'))

watch(
  () => props.settings,
  (settings) => setValues(syncSettingsFormDefaults(settings)),
  { immediate: true }
)

const submitSettings = handleSubmit((values) => {
  emit('update', syncSettingsFormToUpdate(values))
})

function updateBooleanField(event: Event): void {
  const input = event.target as HTMLInputElement
  setFieldValue('sync_enabled', input.checked)
}

function updateNumberField(field: 'batch_size' | 'poll_interval_seconds', event: Event): void {
  const input = event.target as HTMLInputElement
  setFieldValue(field, Number(input.value))
}
</script>

<template>
  <section v-if="settings || isLoading" class="mail-sync-settings-strip" aria-label="Provider sync settings">
    <div class="sync-settings-heading">
      <Icon icon="tabler:refresh-dot" class="sync-settings-icon" />
      <div>
        <div class="sync-settings-title">Provider sync</div>
        <div class="sync-settings-meta">
          <span v-if="isLoading">Loading settings...</span>
          <span v-else>{{ syncStateLabel }}</span>
        </div>
      </div>
    </div>

    <form class="sync-settings-form" @submit.prevent="submitSettings">
      <label class="sync-toggle">
        <input
          :checked="formValues.sync_enabled"
          type="checkbox"
          :disabled="isDisabled"
          @change="updateBooleanField"
        />
        <span>Sync</span>
      </label>

      <label class="sync-field">
        <span>Batch</span>
        <input
          :value="formValues.batch_size"
          type="number"
          min="1"
          max="500"
          step="1"
          :disabled="isDisabled"
          @input="updateNumberField('batch_size', $event)"
        />
        <small v-if="errors.batch_size">{{ errors.batch_size }}</small>
      </label>

      <label class="sync-field">
        <span>Poll, sec</span>
        <input
          :value="formValues.poll_interval_seconds"
          type="number"
          min="60"
          max="86400"
          step="60"
          :disabled="isDisabled"
          @input="updateNumberField('poll_interval_seconds', $event)"
        />
        <small v-if="errors.poll_interval_seconds">{{ errors.poll_interval_seconds }}</small>
      </label>

      <Button variant="outline" size="sm" :disabled="isDisabled" :loading="isSaving" type="submit">
        Save
      </Button>
    </form>
  </section>
</template>

<style scoped>
.mail-sync-settings-strip {
  display: grid;
  grid-template-columns: minmax(10rem, 0.4fr) minmax(0, 1fr);
  gap: 0.75rem;
  align-items: center;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 82%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.sync-settings-heading {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.sync-settings-icon {
  width: 16px;
  height: 16px;
  color: var(--hh-accent, #2563eb);
}

.sync-settings-title {
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
  font-weight: 700;
}

.sync-settings-meta {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
}

.sync-settings-form {
  display: flex;
  align-items: flex-start;
  justify-content: flex-end;
  gap: 0.5rem;
  min-width: 0;
}

.sync-toggle,
.sync-field {
  display: grid;
  gap: 0.1875rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
  font-weight: 600;
}

.sync-toggle {
  grid-template-columns: auto auto;
  align-items: center;
  padding-top: 1.2rem;
}

.sync-toggle input {
  accent-color: var(--hh-accent, #2563eb);
}

.sync-field input {
  width: 6.5rem;
  min-height: 1.75rem;
  border: 1px solid var(--hh-border, #d1d5db);
  border-radius: var(--hh-radius-sm, 0.375rem);
  padding: 0.25rem 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 74%, transparent);
  color: var(--hh-text-primary, #111827);
  font-size: 0.75rem;
}

.sync-field small {
  max-width: 8rem;
  color: var(--hh-text-error, #ef4444);
  font-size: 0.625rem;
  line-height: 1.2;
}

@media (max-width: 900px) {
  .mail-sync-settings-strip {
    grid-template-columns: 1fr;
  }

  .sync-settings-form {
    justify-content: flex-start;
    flex-wrap: wrap;
  }
}
</style>
