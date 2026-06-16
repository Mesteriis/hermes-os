<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useTelegramAccountCapabilitiesQuery } from '../queries/useTelegramQuery'

const { t } = useI18n()

const props = defineProps<{
  accountId: string | null
}>()

const capabilitiesQuery = useTelegramAccountCapabilitiesQuery(computed(() => props.accountId))
const accountScope = computed(() => capabilitiesQuery.data.value?.account_scope ?? null)
const capabilityRows = computed(() => capabilitiesQuery.data.value?.capabilities ?? [])
const unsupportedFeatures = computed(() => capabilitiesQuery.data.value?.unsupported_features ?? [])

function tone(status: string): string {
  if (status === 'available') return 'success'
  if (status === 'degraded') return 'warning'
  if (status === 'blocked') return 'danger'
  return 'muted'
}
</script>

<template>
  <section class="telegram-capability-matrix telegram-rail-card">
    <header class="telegram-capability-matrix__header">
      <div>
        <h3>{{ t('Capabilities') }}</h3>
        <p>{{ t('Account-scoped capability contract for the current Telegram account.') }}</p>
      </div>
    </header>

    <div v-if="!accountId" class="telegram-capability-matrix__state">
      {{ t('Select a Telegram chat/account to inspect capability state.') }}
    </div>
    <div v-else-if="capabilitiesQuery.isLoading.value" class="telegram-capability-matrix__state">
      {{ t('Loading Telegram capabilities...') }}
    </div>
    <div v-else-if="!accountScope" class="telegram-capability-matrix__state">
      {{ t('No account-scoped capability payload is available.') }}
    </div>
    <div v-else class="telegram-capability-matrix__body">
      <dl class="telegram-capability-matrix__scope">
        <div><dt>{{ t('Account') }}</dt><dd>{{ accountScope.account_id }}</dd></div>
        <div><dt>{{ t('Provider') }}</dt><dd>{{ accountScope.provider_kind }}</dd></div>
        <div><dt>{{ t('Runtime') }}</dt><dd>{{ accountScope.runtime_kind }}</dd></div>
        <div><dt>{{ t('Lifecycle') }}</dt><dd>{{ accountScope.lifecycle_state }}</dd></div>
      </dl>

      <div v-if="unsupportedFeatures.length > 0" class="telegram-capability-matrix__unsupported">
        <strong>{{ t('Unsupported Features') }}</strong>
        <p>{{ unsupportedFeatures.join(' · ') }}</p>
      </div>

      <div class="telegram-capability-matrix__list">
        <article v-for="capability in capabilityRows" :key="capability.operation" class="telegram-capability-matrix__item">
          <div class="telegram-capability-matrix__item-head">
            <strong>{{ capability.operation }}</strong>
            <span :data-tone="tone(capability.status)">{{ capability.status }}</span>
          </div>
          <small>{{ capability.category }} · {{ capability.action_class }}</small>
          <p>{{ capability.reason }}</p>
          <small>
            {{ t('Confirm') }}: {{ capability.confirmation_required ? 'yes' : 'no' }}
            · {{ t('Closure gate') }}: {{ capability.closure_gate ? 'yes' : 'no' }}
          </small>
        </article>
      </div>
    </div>
  </section>
</template>

<style scoped>
.telegram-capability-matrix,
.telegram-capability-matrix__body,
.telegram-capability-matrix__list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.telegram-capability-matrix__header p,
.telegram-capability-matrix__state,
.telegram-capability-matrix__item small,
.telegram-capability-matrix__scope dt {
  margin: 2px 0 0;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}

.telegram-capability-matrix__scope {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px 12px;
  margin: 0;
}

.telegram-capability-matrix__scope dd {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--color-text, #333);
  word-break: break-word;
}

.telegram-capability-matrix__unsupported p,
.telegram-capability-matrix__item p {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--color-text, #333);
  word-break: break-word;
}

.telegram-capability-matrix__item {
  padding: 8px;
  border-radius: 8px;
  background: var(--color-surface, #fff);
  border: 1px solid var(--color-border, #e0e0e0);
}

.telegram-capability-matrix__item-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.telegram-capability-matrix__item-head span {
  border-radius: 999px;
  padding: 2px 8px;
  font-size: 11px;
  text-transform: lowercase;
}

.telegram-capability-matrix__item-head span[data-tone='success'] {
  background: #e7f6ec;
  color: #206a3a;
}

.telegram-capability-matrix__item-head span[data-tone='warning'] {
  background: #fff4e5;
  color: #9a5b00;
}

.telegram-capability-matrix__item-head span[data-tone='danger'] {
  background: #fdecea;
  color: #b42318;
}

.telegram-capability-matrix__item-head span[data-tone='muted'] {
  background: #f2f4f7;
  color: #667085;
}
</style>
