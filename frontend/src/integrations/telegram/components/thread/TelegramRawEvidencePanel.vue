<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import { useTelegramRawMessageEvidenceQuery } from '../../queries/useTelegramRawEvidenceQuery'

const { t } = useI18n()

const props = defineProps<{
  messageId: string
  isOpen: boolean
}>()

const rawEvidenceQuery = useTelegramRawMessageEvidenceQuery(
  computed(() => props.messageId),
  computed(() => props.isOpen)
)

const rawRecord = computed(() => rawEvidenceQuery.data.value?.raw_record ?? null)
const payloadPreview = computed(() => stringifyJson(rawRecord.value?.payload ?? null))
const provenancePreview = computed(() => stringifyJson(rawRecord.value?.provenance ?? null))

function stringifyJson(value: Record<string, unknown> | null): string {
  if (!value) return '{}'
  return JSON.stringify(value, null, 2)
}
</script>

<template>
  <div class="telegram-raw-evidence">
    <div v-if="rawEvidenceQuery.isLoading.value" class="telegram-raw-evidence__state">
      {{ t('Loading raw source evidence...') }}
    </div>
    <div v-else-if="rawEvidenceQuery.isError.value" class="telegram-raw-evidence__state">
      {{ t('Raw source evidence is unavailable for this message.') }}
    </div>
    <article v-else-if="rawRecord" class="telegram-raw-evidence__card">
      <header>
        <Icon icon="tabler:file-database" width="14" height="14" />
        <div>
          <strong>{{ t('Raw Source Evidence') }}</strong>
          <small>{{ rawRecord.raw_record_id }}</small>
        </div>
      </header>
      <dl>
        <div>
          <dt>{{ t('Provider') }}</dt>
          <dd>{{ rawRecord.provider_kind }} · {{ rawRecord.provider_account_id }}</dd>
        </div>
        <div>
          <dt>{{ t('Provider Message') }}</dt>
          <dd>{{ rawRecord.provider_message_id }}</dd>
        </div>
        <div v-if="rawRecord.source_uri">
          <dt>{{ t('Source URI') }}</dt>
          <dd>{{ rawRecord.source_uri }}</dd>
        </div>
        <div>
          <dt>{{ t('Ingested') }}</dt>
          <dd>{{ rawRecord.ingested_at }}</dd>
        </div>
      </dl>
      <details>
        <summary>{{ t('Sanitized payload') }}</summary>
        <pre>{{ payloadPreview }}</pre>
      </details>
      <details>
        <summary>{{ t('Provenance') }}</summary>
        <pre>{{ provenancePreview }}</pre>
      </details>
    </article>
  </div>
</template>

<style scoped>
.telegram-raw-evidence,
.telegram-raw-evidence__card {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.telegram-raw-evidence__state {
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}
.telegram-raw-evidence__card {
  padding: 8px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  background: var(--color-surface, #fff);
}
.telegram-raw-evidence__card header {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}
.telegram-raw-evidence__card strong,
.telegram-raw-evidence__card small {
  display: block;
}
.telegram-raw-evidence__card small,
.telegram-raw-evidence__card dt {
  color: var(--color-text-secondary, #777);
}
.telegram-raw-evidence__card dl {
  display: grid;
  gap: 4px;
  margin: 0;
}
.telegram-raw-evidence__card dt,
.telegram-raw-evidence__card dd,
.telegram-raw-evidence__card summary,
.telegram-raw-evidence__card pre {
  font-size: 11px;
}
.telegram-raw-evidence__card dd {
  margin: 0;
  word-break: break-word;
}
.telegram-raw-evidence__card summary {
  cursor: pointer;
  color: var(--color-text, #333);
}
.telegram-raw-evidence__card pre {
  max-height: 220px;
  overflow: auto;
  margin: 6px 0 0;
  padding: 8px;
  border-radius: 6px;
  background: var(--color-bg, #fafafa);
  color: var(--color-text, #333);
  white-space: pre-wrap;
}
</style>
