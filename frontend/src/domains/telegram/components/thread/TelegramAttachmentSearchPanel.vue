<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import AttachmentSearchPanel from '../../../communications/components/AttachmentSearchPanel.vue'

const { t } = useI18n()

const props = defineProps<{
  accountId: string | null | undefined
}>()

const normalizedAccountId = computed(() => props.accountId?.trim() || null)
</script>

<template>
  <section class="telegram-attachment-search-panel">
    <header class="telegram-attachment-search-panel__header">
      <span class="telegram-attachment-search-panel__icon">
        <Icon icon="tabler:database-search" width="17" height="17" />
      </span>
      <div>
        <strong>{{ t('Downloaded attachment search') }}</strong>
        <p>{{ t('Searches Telegram files already persisted in the shared Communication attachment index.') }}</p>
      </div>
    </header>
    <AttachmentSearchPanel :accountId="normalizedAccountId" />
  </section>
</template>

<style scoped>
.telegram-attachment-search-panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 8px 0;
}

.telegram-attachment-search-panel__header {
  display: flex;
  gap: 8px;
  align-items: flex-start;
  padding: 8px 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  background: var(--color-bg-elevated, #fff);
}

.telegram-attachment-search-panel__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 999px;
  color: var(--color-primary, #1976d2);
  background: var(--color-primary-subtle, #e3f2fd);
  flex: 0 0 auto;
}

.telegram-attachment-search-panel__header strong {
  display: block;
  font-size: 12px;
  color: var(--color-text, #222);
}

.telegram-attachment-search-panel__header p {
  margin: 2px 0 0;
  font-size: 11px;
  line-height: 1.35;
  color: var(--color-text-secondary, #777);
}
</style>
