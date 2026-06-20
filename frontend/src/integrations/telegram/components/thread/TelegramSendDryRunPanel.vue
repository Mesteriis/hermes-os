<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../../platform/i18n'
import type { TelegramAutomationPolicy, TelegramAutomationTemplate } from '../../types/automation'
import {
  useTelegramAutomationPoliciesQuery,
  useTelegramAutomationTemplatesQuery,
  useTelegramSendDryRunMutation,
} from '../../queries/useTelegramAutomationQuery'

const { t } = useI18n()

const props = defineProps<{
  accountId: string | null
  providerChatId: string | null
  text: string
}>()

const selectedPolicyId = ref('')
const variableValues = ref<Record<string, string>>({})
const dryRunResult = ref<{ rendered_text: string; rendered_preview_hash: string; status: string } | null>(null)
const dryRunError = ref('')

const policiesQuery = useTelegramAutomationPoliciesQuery(computed(() => props.accountId))
const templatesQuery = useTelegramAutomationTemplatesQuery()
const dryRunMutation = useTelegramSendDryRunMutation()

const eligiblePolicies = computed(() =>
  (policiesQuery.data.value ?? []).filter(
    (policy) =>
      policy.enabled &&
      !!props.providerChatId &&
      policy.allowed_chat_ids.includes(props.providerChatId)
  )
)

const selectedPolicy = computed<TelegramAutomationPolicy | null>(
  () => eligiblePolicies.value.find((policy) => policy.policy_id === selectedPolicyId.value) ?? null
)

const selectedTemplate = computed<TelegramAutomationTemplate | null>(
  () =>
    (templatesQuery.data.value ?? []).find(
      (template) => template.template_id === selectedPolicy.value?.template_id
    ) ?? null
)

const requiredVariables = computed(() => selectedTemplate.value?.required_variables ?? [])
const isBusy = computed(
  () =>
    policiesQuery.isLoading.value ||
    templatesQuery.isLoading.value ||
    dryRunMutation.isPending.value
)

watch(
  eligiblePolicies,
  (policies) => {
    if (!policies.some((policy) => policy.policy_id === selectedPolicyId.value)) {
      selectedPolicyId.value = policies[0]?.policy_id ?? ''
    }
  },
  { immediate: true }
)

watch(requiredVariables, (variables) => {
  const nextValues: Record<string, string> = {}
  for (const variable of variables) {
    nextValues[variable] = variableValues.value[variable] ?? ''
  }
  variableValues.value = nextValues
})

async function runDryRun() {
  if (!selectedPolicy.value || !props.providerChatId) return
  dryRunError.value = ''
  try {
    const response = await dryRunMutation.mutateAsync({
      command_id: `dry-run-${Date.now()}`,
      policy_id: selectedPolicy.value.policy_id,
      provider_chat_id: props.providerChatId,
      variables: variableValues.value,
      source_context: {
        source: 'telegram_workbench',
        composer_text: props.text,
      },
    })
    dryRunResult.value = {
      rendered_text: response.rendered_text,
      rendered_preview_hash: response.rendered_preview_hash,
      status: response.status,
    }
  } catch (error) {
    dryRunResult.value = null
    dryRunError.value = error instanceof Error ? error.message : String(error)
  }
}
</script>

<template>
  <section class="telegram-dry-run-panel">
    <header>
      <strong>{{ t('Send Dry Run') }}</strong>
      <small>{{ t('Preview policy-rendered Telegram output before sending.') }}</small>
    </header>

    <div v-if="isBusy && eligiblePolicies.length === 0" class="telegram-dry-run-panel__state">
      {{ t('Loading Telegram automation policies...') }}
    </div>
    <div v-else-if="!accountId || !providerChatId" class="telegram-dry-run-panel__state">
      {{ t('Select a Telegram chat before running a dry run.') }}
    </div>
    <div v-else-if="eligiblePolicies.length === 0" class="telegram-dry-run-panel__state">
      {{ t('No enabled Telegram send policy allows this chat yet.') }}
    </div>
    <div v-else class="telegram-dry-run-panel__body">
      <label>
        <span>{{ t('Policy') }}</span>
        <select v-model="selectedPolicyId">
          <option v-for="policy in eligiblePolicies" :key="policy.policy_id" :value="policy.policy_id">
            {{ policy.name }}
          </option>
        </select>
      </label>

      <label v-for="variable in requiredVariables" :key="variable">
        <span>{{ variable }}</span>
        <input v-model="variableValues[variable]" type="text" autocomplete="off" />
      </label>

      <button
        type="button"
        :disabled="dryRunMutation.isPending.value || !selectedPolicyId"
        @click="void runDryRun()"
      >
        {{ t('Run Dry Run') }}
      </button>

      <p v-if="dryRunError" class="telegram-dry-run-panel__error">{{ dryRunError }}</p>

      <article v-if="dryRunResult" class="telegram-dry-run-panel__result">
        <strong>{{ dryRunResult.status }}</strong>
        <small>{{ dryRunResult.rendered_preview_hash }}</small>
        <p>{{ dryRunResult.rendered_text }}</p>
      </article>
    </div>
  </section>
</template>

<style scoped>
.telegram-dry-run-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 8px 10px;
  border-top: 1px solid var(--color-border, #e0e0e0);
}

.telegram-dry-run-panel header {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.telegram-dry-run-panel header small,
.telegram-dry-run-panel__state,
.telegram-dry-run-panel__result small {
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}

.telegram-dry-run-panel__body {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.telegram-dry-run-panel__body label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
}

.telegram-dry-run-panel__body select,
.telegram-dry-run-panel__body input {
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 6px;
  padding: 6px 8px;
  font-size: 12px;
  background: var(--color-surface, #fff);
}

.telegram-dry-run-panel__body button {
  align-self: flex-start;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 6px;
  background: var(--color-surface, #fff);
  padding: 6px 10px;
  font-size: 12px;
  cursor: pointer;
}

.telegram-dry-run-panel__error {
  margin: 0;
  font-size: 12px;
  color: var(--color-error-text, #c62828);
}

.telegram-dry-run-panel__result {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px;
  border-radius: 8px;
  background: var(--color-bg, #f5f5f5);
}

.telegram-dry-run-panel__result p {
  margin: 0;
  font-size: 12px;
  color: var(--color-text, #333);
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
