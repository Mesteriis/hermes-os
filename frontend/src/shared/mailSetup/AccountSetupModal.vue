<script setup lang="ts">
import { computed, ref } from 'vue'
import { useForm } from 'vee-validate'
import { loadFrontendConfig } from '../../platform/config/env'
import Icon from '../ui/Icon.vue'
import Button from '../ui/Button.vue'
import Dialog from '../ui/Dialog.vue'
import {
  useSetupImapEmailAccountMutation,
  useStartGmailOAuthSetupMutation
} from '../../integrations/mail/queries/accountSetupQueries'
import {
  accountSetupFormDefaults,
  accountSetupFormToGmailOAuthStart,
  accountSetupFormToImapRequest,
  accountSetupVeeValidationSchema,
  type AccountSetupFormValues,
  type MailAccountSetupProvider
} from '../../integrations/mail/forms/accountSetupForm'

const emit = defineEmits<{
  close: []
}>()

const step = ref(1)
const setupError = ref('')
const setupStatusMessage = ref('')
const frontendConfig = loadFrontendConfig()
const gmailOAuthSetupMutation = useStartGmailOAuthSetupMutation()
const imapEmailAccountSetupMutation = useSetupImapEmailAccountMutation()

const providerOptions: { kind: MailAccountSetupProvider; label: string; icon: string; description: string }[] = [
  { kind: 'gmail', label: 'Gmail', icon: 'tabler:brand-google', description: 'Google OAuth account setup' },
  { kind: 'icloud', label: 'iCloud', icon: 'tabler:brand-apple', description: 'iCloud Mail with app password' },
  { kind: 'imap', label: 'IMAP', icon: 'tabler:mail', description: 'Generic IMAP and SMTP account' }
]

const {
  errors,
  handleSubmit,
  isSubmitting,
  resetForm,
  setFieldValue,
  values: formValues
} = useForm<AccountSetupFormValues>({
  validationSchema: accountSetupVeeValidationSchema,
  initialValues: accountSetupFormDefaults('icloud')
})

const selectedProvider = computed(() => formValues.provider_kind)
const selectedProviderInfo = computed(() =>
  providerOptions.find(p => p.kind === selectedProvider.value)
)
const submitLabel = computed(() => {
  if (isSubmitting.value) return selectedProvider.value === 'gmail' ? 'Starting...' : 'Connecting...'
  return selectedProvider.value === 'gmail' ? 'Continue with Google' : 'Connect Account'
})

function selectProvider(kind: MailAccountSetupProvider) {
  resetForm({ values: accountSetupFormDefaults(kind) })
  setupError.value = ''
  setupStatusMessage.value = ''
  step.value = 2
}

function goBack() {
  if (step.value > 1) {
    step.value--
    setupError.value = ''
    setupStatusMessage.value = ''
  }
}

const submitAccountSetup = handleSubmit(async (values) => {
  setupError.value = ''
  setupStatusMessage.value = ''

  try {
    if (values.provider_kind === 'gmail') {
      const response = await gmailOAuthSetupMutation.mutateAsync(
        accountSetupFormToGmailOAuthStart(values, frontendConfig.apiBaseUrl)
      )
      window.open(response.authorization_url, '_blank', 'noopener,noreferrer')
      setupStatusMessage.value = 'Google authorization opened'
      return
    }

    await imapEmailAccountSetupMutation.mutateAsync(accountSetupFormToImapRequest(values))
    emit('close')
  } catch (e) {
    setupError.value = e instanceof Error ? e.message : 'Setup failed'
  }
})

function updateStringField(
  key: keyof AccountSetupFormValues,
  event: Event
) {
  setFieldValue(key, (event.target as HTMLInputElement).value)
}

function updateNumberField(
  key: keyof AccountSetupFormValues,
  event: Event
) {
  setFieldValue(key, Number((event.target as HTMLInputElement).value))
}

function updateBooleanField(
  key: keyof AccountSetupFormValues,
  event: Event
) {
  setFieldValue(key, (event.target as HTMLInputElement).checked)
}

function handleClose() {
  emit('close')
}
</script>

<template>
  <Dialog :open="true" content-class="account-setup-dialog" @update:open="(open) => { if (!open) handleClose() }">
    <template #header>
      <div class="modal-header">
        <div class="modal-header-left">
          <Button v-if="step > 1" variant="ghost" size="sm" @click="goBack">
            <Icon icon="tabler:arrow-left" />
          </Button>
          <h2 v-if="step === 1">Add Mail Account</h2>
          <h2 v-else-if="step === 2">Configure {{ selectedProviderInfo?.label }}</h2>
        </div>
      </div>
    </template>

    <div class="setup-modal">
      <!-- Step 1: Provider selection -->
      <div v-if="step === 1" class="provider-selection">
        <p class="step-desc">Select a mail provider to connect</p>
        <div class="provider-grid">
          <button
            v-for="provider in providerOptions"
            :key="provider.kind"
            class="provider-card"
            type="button"
            @click="selectProvider(provider.kind)"
          >
            <Icon :icon="provider.icon" class="provider-icon" />
            <span class="provider-label">{{ provider.label }}</span>
            <span class="provider-desc">{{ provider.description }}</span>
          </button>
        </div>
      </div>

      <!-- Step 2: Account details -->
      <div v-else-if="step === 2" class="account-details">
        <p class="step-desc">Enter your {{ selectedProviderInfo?.label }} account details</p>

        <div class="form-fields">
          <div class="field">
            <label>Account Name</label>
            <input
              type="text"
              :value="formValues.display_name"
              placeholder="e.g., Personal Gmail"
              @input="updateStringField('display_name', $event)"
            />
          </div>
          <div class="field">
            <label>Email Address</label>
            <input
              type="email"
              :value="formValues.email"
              placeholder="you@example.com"
              @input="updateStringField('email', $event)"
            />
            <span v-if="errors.email" class="field-error">{{ errors.email }}</span>
          </div>

          <template v-if="selectedProvider === 'imap'">
            <div class="field">
              <label>IMAP Host</label>
              <input
                type="text"
                :value="formValues.imap_host"
                placeholder="imap.example.com"
                @input="updateStringField('imap_host', $event)"
              />
              <span v-if="errors.imap_host" class="field-error">{{ errors.imap_host }}</span>
            </div>
            <div class="field-row">
              <div class="field">
                <label>IMAP Port</label>
                <input
                  type="number"
                  :value="formValues.imap_port"
                  @input="updateNumberField('imap_port', $event)"
                />
              </div>
              <div class="field">
                <label>Username</label>
                <input
                  type="text"
                  :value="formValues.username"
                  placeholder="user@example.com"
                  @input="updateStringField('username', $event)"
                />
              </div>
            </div>
            <div class="field">
              <label>Password</label>
              <input
                type="password"
                :value="formValues.password"
                placeholder="Mailbox password"
                @input="updateStringField('password', $event)"
              />
              <span v-if="errors.password" class="field-error">{{ errors.password }}</span>
            </div>
            <div class="field">
              <label>SMTP Host</label>
              <input
                type="text"
                :value="formValues.smtp_host"
                placeholder="smtp.example.com"
                @input="updateStringField('smtp_host', $event)"
              />
            </div>
            <div class="field-row">
              <div class="field">
                <label>SMTP Port</label>
                <input
                  type="number"
                  :value="formValues.smtp_port"
                  @input="updateNumberField('smtp_port', $event)"
                />
              </div>
              <div class="field checkbox-field">
                <label class="checkbox-label">
                  <input
                    type="checkbox"
                    :checked="formValues.imap_tls"
                    @change="updateBooleanField('imap_tls', $event)"
                  />
                  IMAP TLS
                </label>
              </div>
            </div>
            <div class="field-row">
              <div class="field checkbox-field">
                <label class="checkbox-label">
                  <input
                    type="checkbox"
                    :checked="formValues.smtp_tls"
                    @change="updateBooleanField('smtp_tls', $event)"
                  />
                  SMTP TLS
                </label>
              </div>
              <div class="field checkbox-field">
                <label class="checkbox-label">
                  <input
                    type="checkbox"
                    :checked="formValues.smtp_starttls"
                    @change="updateBooleanField('smtp_starttls', $event)"
                  />
                  SMTP STARTTLS
                </label>
              </div>
            </div>
          </template>

          <div v-if="selectedProvider === 'icloud'" class="field">
            <label>App Password</label>
            <input
              type="password"
              :value="formValues.password"
              placeholder="Your app-specific password"
              @input="updateStringField('password', $event)"
            />
            <span v-if="errors.password" class="field-error">{{ errors.password }}</span>
          </div>
        </div>

        <div v-if="setupError" class="setup-error">{{ setupError }}</div>
        <div v-if="setupStatusMessage" class="setup-status">{{ setupStatusMessage }}</div>

        <div class="form-actions">
          <Button variant="default" @click="submitAccountSetup" :loading="isSubmitting">
            {{ submitLabel }}
          </Button>
          <Button variant="ghost" @click="goBack">Back</Button>
        </div>
      </div>
    </div>
  </Dialog>
</template>

<style scoped>
:deep(.account-setup-dialog) {
  max-width: 520px;
}

.setup-modal {
  width: 100%;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-right: 2.5rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.modal-header-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.modal-header-left h2 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
}

.step-desc {
  margin: 0 0 1rem;
  font-size: 0.875rem;
  color: var(--hh-text-secondary, #6b7280);
}

.provider-selection,
.account-details {
  padding: 1rem;
  overflow-y: auto;
}

.provider-grid {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.provider-card {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  width: 100%;
  padding: 0.75rem 1rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.5rem;
  background: transparent;
  color: inherit;
  cursor: pointer;
  font: inherit;
  text-align: left;
  transition: background 0.1s, border-color 0.1s;
}

.provider-card:hover {
  background: var(--hh-bg-hover, #f3f4f6);
  border-color: var(--hh-accent, #3b82f6);
}

.provider-icon {
  width: 28px;
  height: 28px;
  color: var(--hh-accent, #3b82f6);
  flex-shrink: 0;
}

.provider-label {
  font-weight: 500;
  font-size: 0.875rem;
  color: var(--hh-text-primary, #1f2937);
}

.provider-desc {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
  margin-left: auto;
}

.form-fields {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.field label {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--hh-text-secondary, #6b7280);
}

.field input[type="text"],
.field input[type="email"],
.field input[type="password"],
.field input[type="number"] {
  padding: 0.5rem 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  font-size: 0.8125rem;
  color: var(--hh-text-primary, #1f2937);
  background: var(--hh-bg-primary, #ffffff);
  outline: none;
}

.field input:focus {
  border-color: var(--hh-accent, #3b82f6);
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.field-row {
  display: flex;
  gap: 0.75rem;
}

.field-row .field {
  flex: 1;
}

.checkbox-field {
  justify-content: flex-end;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.8125rem;
  color: var(--hh-text-primary, #1f2937);
  cursor: pointer;
}

.field-hint {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
  margin: 0.25rem 0 0;
}

.field-error {
  font-size: 0.6875rem;
  color: var(--hh-text-error, #ef4444);
}

.setup-error {
  padding: 0.5rem 0.75rem;
  background: var(--hh-bg-error-light, #fef2f2);
  color: var(--hh-text-error, #ef4444);
  border-radius: 0.375rem;
  font-size: 0.8125rem;
  margin-top: 0.5rem;
}

.setup-status {
  padding: 0.5rem 0.75rem;
  background: var(--hh-bg-success-light, rgba(16, 185, 129, 0.12));
  color: var(--hh-text-success, #059669);
  border-radius: 0.375rem;
  font-size: 0.8125rem;
  margin-top: 0.5rem;
}

.form-actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 1rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--hh-border, #e5e7eb);
}
</style>
