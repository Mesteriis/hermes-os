<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { Organization } from '../types/organization'

const { t } = useI18n()

defineProps<{
  organizations: Organization[]
  selectedOrganizationId: string
  isOrganizationsLoading: boolean
}>()

const emit = defineEmits<{
  selectOrg: [id: string]
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="organizations-list">
    <section class="panel org-list-panel">
      <header class="panel-title-row">
        <h2>{{ t('All Companies') }} ({{ organizations.length }})</h2>
      </header>
      <div v-if="isOrganizationsLoading && organizations.length === 0" class="graph-strip-message">
        <span>{{ t('Loading companies.') }}</span>
      </div>
      <div v-else-if="organizations.length === 0" class="graph-strip-message">
        <span>{{ t('No companies yet.') }}</span>
      </div>
      <template v-else>
        <button
          v-for="org in organizations"
          :key="org.organization_id"
          type="button"
          class="org-row"
          :class="{ active: selectedOrganizationId === org.organization_id }"
          @click="emit('selectOrg', org.organization_id)"
        >
          <span class="round-icon blue"><Icon icon="tabler:building" :size="20" /></span>
          <div>
            <strong>{{ org.display_name }}</strong>
            <p>{{ org.industry || t('Unknown industry') }}{{ org.country ? ` · ${org.country}` : '' }}</p>
          </div>
          <small>{{ org.status }}{{ org.watchlist ? ` · ⚠ ${t('watchlist')}` : '' }}</small>
        </button>
      </template>
    </section>
  </div>
</template>

<style scoped>
.org-list-panel {
  padding: 12px;
}
.org-row {
  display: grid;
  grid-template-columns: 44px 1fr auto;
  gap: 10px;
  align-items: center;
  width: 100%;
  min-height: var(--hh-widget-card-compact);
  border: 1px solid transparent;
  border-radius: var(--hh-radius-md);
  background: transparent;
  color: #e6f7f5;
  padding: 9px 10px;
  text-align: left;
  cursor: pointer;
}
.org-row.active {
  border-color: rgba(45, 240, 206, 0.24);
  background: rgba(25, 109, 100, 0.24);
}
.org-row strong {
  display: block;
  color: var(--hh-color-text-bright);
  font-size: 14px;
  font-weight: 560;
}
.org-row p,
.org-row small {
  display: block;
  margin-top: 5px;
  overflow: hidden;
  font-size: 11px;
  font-style: normal;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
