<script setup lang="ts">
import { computed, ref } from 'vue'
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import { usePersonasQuery } from '../queries/useCommunicationsQuery'

const emit = defineEmits<{
  apply: [signature: string]
}>()

const personasQuery = usePersonasQuery()
const selectedPersonaId = ref('')

const personasWithSignatures = computed(() => {
  return (personasQuery.data.value ?? []).filter((persona) => persona.signature.trim())
})
const selectedPersona = computed(() => {
  return personasWithSignatures.value.find((persona) => persona.persona_id === selectedPersonaId.value) ?? null
})
const isLoading = computed(() => personasQuery.isPending.value)
const canApplySignature = computed(() => Boolean(selectedPersona.value?.signature.trim()))

function applySignature(): void {
  const signature = selectedPersona.value?.signature.trim()
  if (!signature) return
  emit('apply', signature)
}
</script>

<template>
  <section class="compose-signature-picker" aria-label="Email signatures">
    <label>
      <span>Signature</span>
      <select
        :value="selectedPersonaId"
        :disabled="isLoading || personasWithSignatures.length === 0"
        @change="selectedPersonaId = ($event.target as HTMLSelectElement).value"
      >
        <option value="">
          {{ isLoading ? 'Loading signatures...' : 'No signature' }}
        </option>
        <option
          v-for="persona in personasWithSignatures"
          :key="persona.persona_id"
          :value="persona.persona_id"
        >
          {{ persona.display_name || persona.name }}
        </option>
      </select>
    </label>
    <Button
      variant="secondary"
      size="sm"
      :disabled="!canApplySignature"
      @click="applySignature"
    >
      <Icon icon="tabler:signature" size="16" />
      Insert
    </Button>
  </section>
</template>

<style scoped>
.compose-signature-picker {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.5rem;
  align-items: end;
  padding: 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-secondary, #f9fafb) 82%, transparent);
}

.compose-signature-picker label {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 0.25rem;
}

.compose-signature-picker span {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
  font-weight: 500;
}

.compose-signature-picker select {
  min-width: 0;
  padding: 0.4375rem 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: var(--hh-bg-primary, #ffffff);
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
  outline: none;
}

.compose-signature-picker select:focus {
  border-color: var(--hh-accent, #3b82f6);
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}
</style>
