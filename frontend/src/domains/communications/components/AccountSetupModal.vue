<script setup lang="ts">
import { ref, computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'

type ProviderKind = 'gmail' | 'icloud' | 'imap'

const emit = defineEmits<{
  close: []
}>()

const step = ref(1)
const selectedProvider = ref<ProviderKind | null>(null)
const accountName = ref('')
const email = ref('')
const appPassword = ref('')
const imapHost = ref('')
const imapPort = ref(993)
const smtpHost = ref('')
const smtpPort = ref(587)
const useSSL = ref(true)
const isSubmitting = ref(false)
const setupError = ref('')

const providerOptions: { kind: ProviderKind; label: string; icon: string; description: string }[] = [
  { kind: 'gmail', label: 'Gmail', icon: 'tabler:brand-google', description: 'Connect a Gmail or Google Workspace account' },
  { kind: 'icloud', label: 'iCloud', icon: 'tabler:brand-apple', description: 'Connect an iCloud Mail account' },
  { kind: 'imap', label: 'IMAP', icon: 'tabler:mail', description: 'Connect any IMAP/SMTP email account' }
]

const selectedProviderInfo = computed(() =>
  providerOptions.find(p => p.kind === selectedProvider.value)
)

function selectProvider(kind: ProviderKind) {
  selectedProvider.value = kind
  step.value = 2
}

function goBack() {
  if (step.value > 1) {
    step.value--
    setupError.value = ''
  }
}

async function handleSubmit() {
  if (!selectedProvider.value || !email.value) return
  isSubmitting.value = true
  setupError.value = ''

  // Build payload
  const payload: Record<string, unknown> = {
    provider_kind: selectedProvider.value,
    account_name: accountName.value || email.value.split('@')[0],
    email: email.value
  }

  if (selectedProvider.value === 'imap') {
    payload.imap_host = imapHost.value
    payload.imap_port = imapPort.value
    payload.smtp_host = smtpHost.value
    payload.smtp_port = smtpPort.value
    payload.use_ssl = useSSL.value
  }

  // In a real implementation, this would call the API
  // For now, simulate success after a short delay
  try {
    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 1000))
    emit('close')
  } catch (e) {
    setupError.value = e instanceof Error ? e.message : 'Setup failed'
  } finally {
    isSubmitting.value = false
  }
}

function handleClose() {
  emit('close')
}
</script>

<template>
  <div class="modal-overlay" @click.self="handleClose">
    <div class="setup-modal">
      <!-- Header -->
      <div class="modal-header">
        <div class="modal-header-left">
          <Button v-if="step > 1" variant="ghost" size="sm" @click="goBack">
            <Icon icon="tabler:arrow-left" />
          </Button>
          <h2 v-if="step === 1">Add Mail Account</h2>
          <h2 v-else-if="step === 2">Configure {{ selectedProviderInfo?.label }}</h2>
        </div>
        <Button variant="ghost" size="sm" @click="handleClose">
          <Icon icon="tabler:x" />
        </Button>
      </div>

      <!-- Step 1: Provider selection -->
      <div v-if="step === 1" class="provider-selection">
        <p class="step-desc">Select a mail provider to connect</p>
        <div class="provider-grid">
          <div
            v-for="provider in providerOptions"
            :key="provider.kind"
            class="provider-card"
            @click="selectProvider(provider.kind)"
          >
            <Icon :icon="provider.icon" class="provider-icon" />
            <span class="provider-label">{{ provider.label }}</span>
            <span class="provider-desc">{{ provider.description }}</span>
          </div>
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
              v-model="accountName"
              placeholder="e.g., Personal Gmail"
            />
          </div>
          <div class="field">
            <label>Email Address</label>
            <input
              type="email"
              v-model="email"
              placeholder="you@example.com"
            />
          </div>

          <!-- IMAP specific fields -->
          <template v-if="selectedProvider === 'imap'">
            <div class="field">
              <label>IMAP Host</label>
              <input
                type="text"
                v-model="imapHost"
                placeholder="imap.example.com"
              />
            </div>
            <div class="field-row">
              <div class="field">
                <label>IMAP Port</label>
                <input type="number" v-model.number="imapPort" />
              </div>
              <div class="field">
                <label>SMTP Host</label>
                <input
                  type="text"
                  v-model="smtpHost"
                  placeholder="smtp.example.com"
                />
              </div>
            </div>
            <div class="field-row">
              <div class="field">
                <label>SMTP Port</label>
                <input type="number" v-model.number="smtpPort" />
              </div>
              <div class="field checkbox-field">
                <label class="checkbox-label">
                  <input type="checkbox" v-model="useSSL" />
                  Use SSL
                </label>
              </div>
            </div>
          </template>

          <!-- App Password (shown for Gmail/iCloud) -->
          <div v-if="selectedProvider !== 'imap'" class="field">
            <label>App Password</label>
            <input
              type="password"
              v-model="appPassword"
              placeholder="Your app-specific password"
            />
            <p class="field-hint">
              Generate an app password from your {{ selectedProviderInfo?.label }} account settings.
              Your regular password will not work.
            </p>
          </div>
        </div>

        <div v-if="setupError" class="setup-error">{{ setupError }}</div>

        <div class="form-actions">
          <Button variant="default" @click="handleSubmit" :disabled="isSubmitting || !email">
            {{ isSubmitting ? 'Connecting...' : 'Connect Account' }}
          </Button>
          <Button variant="ghost" @click="goBack">Back</Button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
}

.setup-modal {
  width: 480px;
  max-height: 85vh;
  background: var(--hh-bg-primary, #ffffff);
  border-radius: 0.75rem;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
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
  padding: 0.75rem 1rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.5rem;
  cursor: pointer;
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

.setup-error {
  padding: 0.5rem 0.75rem;
  background: var(--hh-bg-error-light, #fef2f2);
  color: var(--hh-text-error, #ef4444);
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
