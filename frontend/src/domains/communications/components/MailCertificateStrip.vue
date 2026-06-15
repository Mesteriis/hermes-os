<script setup lang="ts">
import { computed, ref } from 'vue'
import { useForm } from 'vee-validate'
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import {
  certificateFormDefaults,
  certificateFormToCreateRequest,
  certificateVeeValidationSchema,
  type CertificateFormValues
} from '../forms/certificateForm'
import {
  useCreateMailCertificateMutation,
  useExpiringMailCertificatesQuery,
  useMailCertificatesQuery
} from '../queries/useCommunicationsQuery'
import {
  certificateProviderOptions,
  certificateStorageKindOptions,
  certificateTrustStatusOptions,
  certificateTypeOptions,
  type MailCertificate
} from '../types/certificates'

const isOpen = ref(false)
const statusMessage = ref('')
const errorMessage = ref('')
const certificatesQuery = useMailCertificatesQuery()
const expiringQuery = useExpiringMailCertificatesQuery(90)
const createCertificateMutation = useCreateMailCertificateMutation()

const {
  errors,
  handleSubmit,
  resetForm,
  setFieldValue,
  values: formValues
} = useForm<CertificateFormValues>({
  validationSchema: certificateVeeValidationSchema,
  initialValues: certificateFormDefaults()
})

const certificates = computed(() => certificatesQuery.data.value ?? [])
const expiringCertificates = computed(() => expiringQuery.data.value ?? [])
const isLoading = computed(() => certificatesQuery.isLoading.value || expiringQuery.isLoading.value)
const isSaving = computed(() => createCertificateMutation.isPending.value)
const visibleCertificates = computed(() => certificates.value.slice(0, 3))
const visibleExpiringCertificates = computed(() => expiringCertificates.value.slice(0, 3))

const submitCertificate = handleSubmit(async (values) => {
  errorMessage.value = ''
  statusMessage.value = ''
  try {
    await createCertificateMutation.mutateAsync(certificateFormToCreateRequest(values))
    statusMessage.value = 'Certificate metadata saved'
    resetForm({ values: certificateFormDefaults() })
  } catch (e) {
    errorMessage.value = e instanceof Error ? e.message : 'Certificate save failed'
  }
})

function toggleOpen(): void {
  isOpen.value = !isOpen.value
}

function updateTextField(field: keyof CertificateFormValues, event: Event): void {
  const input = event.target as HTMLInputElement | HTMLSelectElement
  setFieldValue(field, input.value)
}

function certificateLabel(certificate: MailCertificate): string {
  return `${certificate.owner_name} · ${certificate.trust_status}`
}
</script>

<template>
  <section class="mail-certificate-strip" aria-label="Mail certificates">
    <button class="certificate-toggle" type="button" :aria-expanded="isOpen" @click="toggleOpen">
      <span class="certificate-title">
        <Icon icon="tabler:certificate" />
        Certificates
      </span>
      <span class="certificate-count">
        {{ isLoading ? 'Loading...' : `${certificates.length} stored · ${expiringCertificates.length} expiring` }}
      </span>
    </button>

    <div v-if="isOpen" class="certificate-body">
      <div class="certificate-groups">
        <article class="certificate-group">
          <div class="certificate-heading">Expiring certificates</div>
          <div v-if="isLoading" class="certificate-muted">Loading certificates...</div>
          <div v-else-if="visibleExpiringCertificates.length === 0" class="certificate-muted">No expiry in 90 days</div>
          <div v-else class="certificate-list">
            <span
              v-for="certificate in visibleExpiringCertificates"
              :key="certificate.cert_id"
              class="certificate-chip warning"
            >
              {{ certificate.owner_name }} · {{ certificate.valid_until ?? 'unknown expiry' }}
            </span>
          </div>
        </article>

        <article class="certificate-group">
          <div class="certificate-heading">Stored certificates</div>
          <div v-if="isLoading" class="certificate-muted">Loading certificates...</div>
          <div v-else-if="visibleCertificates.length === 0" class="certificate-muted">No certificate metadata</div>
          <div v-else class="certificate-list">
            <span
              v-for="certificate in visibleCertificates"
              :key="certificate.cert_id"
              class="certificate-chip"
              :class="{ warning: certificate.is_revoked || certificate.trust_status !== 'trusted' }"
            >
              {{ certificateLabel(certificate) }}
            </span>
          </div>
        </article>
      </div>

      <form class="certificate-form" @submit.prevent="submitCertificate">
        <div class="certificate-form-title">Add certificate</div>
        <label>
          <span>Certificate id</span>
          <input :value="formValues.cert_id" type="text" autocomplete="off" @input="updateTextField('cert_id', $event)" />
          <small v-if="errors.cert_id">{{ errors.cert_id }}</small>
        </label>
        <label>
          <span>Owner</span>
          <input :value="formValues.owner_name" type="text" autocomplete="off" @input="updateTextField('owner_name', $event)" />
          <small v-if="errors.owner_name">{{ errors.owner_name }}</small>
        </label>
        <label>
          <span>Issuer</span>
          <input :value="formValues.issuer" type="text" autocomplete="off" @input="updateTextField('issuer', $event)" />
          <small v-if="errors.issuer">{{ errors.issuer }}</small>
        </label>
        <label>
          <span>Fingerprint SHA-256</span>
          <input :value="formValues.fingerprint_sha256" type="text" autocomplete="off" @input="updateTextField('fingerprint_sha256', $event)" />
        </label>
        <label>
          <span>Valid until</span>
          <input :value="formValues.valid_until" type="datetime-local" @input="updateTextField('valid_until', $event)" />
        </label>
        <label>
          <span>Type</span>
          <select :value="formValues.cert_type" @change="updateTextField('cert_type', $event)">
            <option v-for="option in certificateTypeOptions" :key="option" :value="option">{{ option }}</option>
          </select>
        </label>
        <label>
          <span>Provider</span>
          <select :value="formValues.provider" @change="updateTextField('provider', $event)">
            <option v-for="option in certificateProviderOptions" :key="option" :value="option">{{ option }}</option>
          </select>
        </label>
        <label>
          <span>Storage</span>
          <select :value="formValues.storage_kind" @change="updateTextField('storage_kind', $event)">
            <option v-for="option in certificateStorageKindOptions" :key="option" :value="option">{{ option }}</option>
          </select>
        </label>
        <label>
          <span>Storage reference</span>
          <input :value="formValues.storage_ref" type="text" autocomplete="off" placeholder="keychain item, vault ref, token id" @input="updateTextField('storage_ref', $event)" />
        </label>
        <label>
          <span>Trust</span>
          <select :value="formValues.trust_status" @change="updateTextField('trust_status', $event)">
            <option v-for="option in certificateTrustStatusOptions" :key="option" :value="option">{{ option }}</option>
          </select>
        </label>
        <label>
          <span>Usage</span>
          <input :value="formValues.usage" type="text" autocomplete="off" placeholder="signing, encryption" @input="updateTextField('usage', $event)" />
        </label>

        <p v-if="statusMessage" class="certificate-status">{{ statusMessage }}</p>
        <p v-if="errorMessage" class="certificate-error">{{ errorMessage }}</p>
        <Button variant="outline" size="sm" type="submit" :loading="isSaving">
          Save metadata
        </Button>
      </form>
    </div>
  </section>
</template>

<style scoped>
.mail-certificate-strip {
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 84%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.certificate-toggle {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  border: 0;
  padding: 0.5rem 0.75rem;
  background: transparent;
  color: var(--hh-text-primary, #1f2937);
  cursor: pointer;
}

.certificate-title,
.certificate-count {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  min-width: 0;
  font-size: 0.75rem;
}

.certificate-title {
  font-weight: 700;
}

.certificate-count {
  color: var(--hh-text-secondary, #6b7280);
}

.certificate-body {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(18rem, 0.85fr);
  gap: 0.75rem;
  padding: 0 0.75rem 0.75rem;
}

.certificate-groups,
.certificate-group,
.certificate-form {
  display: grid;
  gap: 0.5rem;
  min-width: 0;
}

.certificate-heading,
.certificate-form-title {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.certificate-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.certificate-chip,
.certificate-muted {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
}

.certificate-chip {
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 999px;
  padding: 0.125rem 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 72%, transparent);
}

.certificate-chip.warning {
  color: var(--hh-text-error, #ef4444);
}

.certificate-form {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  align-items: end;
}

.certificate-form-title,
.certificate-status,
.certificate-error {
  grid-column: 1 / -1;
}

.certificate-form label {
  display: grid;
  gap: 0.1875rem;
  min-width: 0;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
  font-weight: 600;
}

.certificate-form input,
.certificate-form select {
  min-height: 1.875rem;
  border: 1px solid var(--hh-border, #d1d5db);
  border-radius: var(--hh-radius-sm, 0.375rem);
  padding: 0.25rem 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 74%, transparent);
  color: var(--hh-text-primary, #111827);
  font-size: 0.75rem;
}

.certificate-form small,
.certificate-error {
  color: var(--hh-text-error, #ef4444);
  font-size: 0.625rem;
}

.certificate-status {
  color: var(--hh-text-success, #16a34a);
  font-size: 0.6875rem;
}

@media (max-width: 900px) {
  .certificate-body,
  .certificate-form {
    grid-template-columns: 1fr;
  }
}
</style>
